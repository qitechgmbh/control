use super::AngleKind;

quantity! {
    /// Angle (dimensionless quantity).
    quantity: Angle; "angle";
    /// Dimension of angle, 1 (dimensionless).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        Z0,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    kind: dyn AngleKind;
    units {
        @radian: 1.0; "rad", "radian", "radians";
        @degree: 1.745_329_251_994_329_5e-2; "°", "degree", "degrees";
        @revolution: 6.283_185_307_179_586; "r", "revolution", "revolutions";
    }
}
