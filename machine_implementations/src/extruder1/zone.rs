/// Which extruder the zones belong to.
///
/// The two generations carry different heater bands, so anything quoted in
/// watts has to say which machine it means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Generation {
    /// `MACHINE_EXTRUDER_V1`.
    V1,
    /// `MACHINE_EXTRUDER_V2`, the one the roles call V3.
    V2,
}

/// One of the four independently controlled heating zones.
///
/// [`Zone::port`] is the EL3204 / EL2004 port the zone is wired to, and is the
/// index used by every per-zone array in this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Zone {
    Front,
    Middle,
    Back,
    Nozzle,
}

impl Zone {
    /// All zones in port order.
    pub const ALL: [Self; 4] = [Self::Front, Self::Middle, Self::Back, Self::Nozzle];

    /// EL3204 / EL2004 port index.
    pub const fn port(self) -> usize {
        match self {
            Self::Front => 0,
            Self::Middle => 1,
            Self::Back => 2,
            Self::Nozzle => 3,
        }
    }

    /// Lowercase wire-protocol name, as used by `extruder1::api`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Front => "front",
            Self::Middle => "middle",
            Self::Back => "back",
            Self::Nozzle => "nozzle",
        }
    }

    /// Inverse of [`Self::name`].
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|z| z.name() == name)
    }

    /// Rated electrical power of the zone's heater band, in W.
    ///
    /// The generations are wired with different bands: V1 runs 700 W on the
    /// barrel and 200 W on the nozzle, V2 runs 900 W and 150 W. Both modules
    /// carried their own figures until the two were merged into one machine
    /// implementation, at which point V2 silently inherited V1's.
    ///
    /// This only sets the reported power (`duty * rated_w`) and the kWh
    /// counter. Every control loop here works in duty, so none of them reads
    /// it.
    pub const fn rated_w(self, generation: Generation) -> f64 {
        match (generation, self) {
            (Generation::V1, Self::Front | Self::Middle | Self::Back) => 700.0,
            (Generation::V1, Self::Nozzle) => 200.0,
            (Generation::V2, Self::Front | Self::Middle | Self::Back) => 900.0,
            (Generation::V2, Self::Nozzle) => 150.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_are_the_index_into_all() {
        for (i, zone) in Zone::ALL.into_iter().enumerate() {
            assert_eq!(zone.port(), i);
        }
    }

    #[test]
    fn names_round_trip() {
        for zone in Zone::ALL {
            assert_eq!(Zone::from_name(zone.name()), Some(zone));
        }
        assert_eq!(Zone::from_name("nope"), None);
    }

    /// The generations really are wired differently, and a refactor that
    /// flattens them back into one set of numbers is a regression — it already
    /// happened once, when the two extruder modules were merged.
    #[test]
    fn each_generation_keeps_its_own_heater_ratings() {
        for zone in [Zone::Front, Zone::Middle, Zone::Back] {
            assert_eq!(zone.rated_w(Generation::V1), 700.0);
            assert_eq!(zone.rated_w(Generation::V2), 900.0);
        }
        assert_eq!(Zone::Nozzle.rated_w(Generation::V1), 200.0);
        assert_eq!(Zone::Nozzle.rated_w(Generation::V2), 150.0);
    }
}
