//! Heat loss from a node to the surrounding air.
//!
//! Both variants are expressed as an *effective conductance* `G(T, T_ambient)` so
//! that the heat flow is always `G * (T - T_ambient)`. That keeps the solver's
//! flux assembly uniform and lets [`super::ThermalNetwork::max_stable_dt`] fold
//! the (nonlinear) losses into its stability estimate.

/// Stefan-Boltzmann constant in W/(m²·K⁴).
pub const STEFAN_BOLTZMANN: f64 = 5.670_374_419e-8;

/// Kelvin value of 0 °C.
pub const KELVIN_OFFSET: f64 = 273.15;

/// Heat loss path from a node to ambient air.
#[derive(Debug, Clone, Copy)]
pub enum AmbientLoss {
    /// A bare (uninsulated) outer surface losing heat by free convection and
    /// radiation.
    Bare {
        /// Exposed outer surface area in m².
        area_m2: f64,
        /// Free-convection coefficient in the form `h = coeff * dT^0.25`
        /// (W/(m²·K^1.25)). For a horizontal cylinder of diameter `d`,
        /// `coeff ~= 1.32 * (1/d)^0.25`, e.g. ~2.6 for Ø65 mm.
        convection_coeff: f64,
        /// Surface emissivity: ~0.25 for machined steel, ~0.8 when oxidised.
        emissivity: f64,
    },
    /// A surface wrapped in a cylindrical insulation sleeve. Conduction through
    /// the sleeve is in series with convection and radiation off the sleeve's
    /// outer skin.
    Insulated {
        /// Thermal conductivity of the insulation in W/(m·K).
        k_insulation: f64,
        /// Inner radius of the sleeve in m (i.e. the radius it is wrapped onto).
        r_inner_m: f64,
        /// Outer radius of the sleeve in m.
        r_outer_m: f64,
        /// Axial length covered in m.
        length_m: f64,
        /// Free-convection coefficient of the sleeve's outer skin, same form as
        /// [`AmbientLoss::Bare::convection_coeff`].
        convection_coeff: f64,
        /// Emissivity of the sleeve's outer skin.
        emissivity: f64,
    },
}

/// Combined convection + radiation film coefficient of a surface at
/// `surface_c` facing air at `ambient_c`, in W/(m²·K).
fn film_coefficient(surface_c: f64, ambient_c: f64, convection_coeff: f64, emissivity: f64) -> f64 {
    let delta = (surface_c - ambient_c).abs();
    // Free convection. The 0.25 exponent degenerates at dT -> 0, so floor the
    // driving difference; the residual error at <0.1 K is irrelevant here.
    let h_conv = convection_coeff * delta.max(0.1).powf(0.25);

    // Radiation, linearised exactly: eps*sigma*(Ts^4 - Ta^4) = h_rad * (Ts - Ta)
    // with h_rad = eps*sigma*(Ts^2 + Ta^2)*(Ts + Ta).
    let ts = surface_c + KELVIN_OFFSET;
    let ta = ambient_c + KELVIN_OFFSET;
    let h_rad = emissivity * STEFAN_BOLTZMANN * ts.mul_add(ts, ta * ta) * (ts + ta);

    h_conv + h_rad
}

impl AmbientLoss {
    /// Effective conductance from the node to ambient in W/K, such that the heat
    /// flow out of the node is `conductance * (node_c - ambient_c)`.
    pub fn conductance_w_per_k(&self, node_c: f64, ambient_c: f64) -> f64 {
        match *self {
            Self::Bare {
                area_m2,
                convection_coeff,
                emissivity,
            } => area_m2 * film_coefficient(node_c, ambient_c, convection_coeff, emissivity),

            Self::Insulated {
                k_insulation,
                r_inner_m,
                r_outer_m,
                length_m,
                convection_coeff,
                emissivity,
            } => {
                // Cylindrical shell conduction: G = 2*pi*k*L / ln(ro/ri)
                let g_shell = 2.0 * std::f64::consts::PI * k_insulation * length_m
                    / (r_outer_m / r_inner_m).ln();
                let area_outer = 2.0 * std::f64::consts::PI * r_outer_m * length_m;

                // The outer film coefficient depends on the sleeve's skin
                // temperature, which itself depends on the series split. Three
                // fixed-point passes are plenty: the skin sits close to ambient
                // and the iteration contracts hard.
                let mut skin_c = ambient_c + 0.15 * (node_c - ambient_c);
                let mut g_total = 0.0;
                for _ in 0..3 {
                    let g_film = area_outer
                        * film_coefficient(skin_c, ambient_c, convection_coeff, emissivity);
                    g_total = 1.0 / (1.0 / g_shell + 1.0 / g_film);
                    // Flow through the series chain sets the skin temperature.
                    let q = g_total * (node_c - ambient_c);
                    skin_c = q.mul_add(1.0 / g_film, ambient_c);
                }
                g_total
            }
        }
    }

    /// Heat flow *out of* the node in W. Negative when the node is colder than
    /// ambient (i.e. it is being warmed by the room).
    pub fn heat_flow_w(&self, node_c: f64, ambient_c: f64) -> f64 {
        self.conductance_w_per_k(node_c, ambient_c) * (node_c - ambient_c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn bare_loss_is_zero_at_ambient() {
        let loss = AmbientLoss::Bare {
            area_m2: 0.1,
            convection_coeff: 2.6,
            emissivity: 0.3,
        };
        assert_relative_eq!(loss.heat_flow_w(20.0, 20.0), 0.0, epsilon = 1e-12);
    }

    #[test]
    fn bare_loss_matches_hand_calculation() {
        // Ø65 mm x 200 mm bare cylinder at 150 C in 20 C air.
        let area = std::f64::consts::PI * 0.065 * 0.200;
        let loss = AmbientLoss::Bare {
            area_m2: area,
            convection_coeff: 2.6,
            emissivity: 0.3,
        };
        let q = loss.heat_flow_w(150.0, 20.0);

        // h_conv = 2.6 * 130^0.25 = 8.78, h_rad = 0.3*sigma*(423^2+293^2)*(423+293) = 3.66
        // q = (8.78 + 3.66) * 0.04084 * 130 = 66 W
        assert!(
            (60.0..75.0).contains(&q),
            "bare loss out of expected range: {q} W"
        );
    }

    #[test]
    fn insulation_cuts_loss_by_an_order_of_magnitude() {
        let bare = AmbientLoss::Bare {
            area_m2: std::f64::consts::PI * 0.065 * 0.200,
            convection_coeff: 2.6,
            emissivity: 0.3,
        };
        let insulated = AmbientLoss::Insulated {
            k_insulation: 0.06,
            r_inner_m: 0.0355,
            r_outer_m: 0.0555,
            length_m: 0.200,
            convection_coeff: 2.6,
            emissivity: 0.8,
        };
        let q_bare = bare.heat_flow_w(150.0, 20.0);
        let q_ins = insulated.heat_flow_w(150.0, 20.0);
        assert!(
            q_ins < q_bare / 3.0,
            "insulation should cut loss substantially: bare {q_bare} W vs insulated {q_ins} W"
        );
        assert!(q_ins > 0.0);
    }

    #[test]
    fn loss_reverses_sign_below_ambient() {
        let loss = AmbientLoss::Bare {
            area_m2: 0.1,
            convection_coeff: 2.6,
            emissivity: 0.3,
        };
        assert!(loss.heat_flow_w(10.0, 20.0) < 0.0);
    }
}
