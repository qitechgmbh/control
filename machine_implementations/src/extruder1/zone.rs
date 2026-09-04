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

/// Serialises as the lowercase wire-protocol name from [`Zone::name`], the
/// same form `extruder1::api` already uses on the wire.
impl serde::Serialize for Zone {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl<'de> serde::Deserialize<'de> for Zone {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let name = <std::borrow::Cow<'de, str>>::deserialize(deserializer)?;
        Self::from_name(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown zone {name:?}")))
    }
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
    pub const fn rated_w(self) -> f64 {
        match self {
            Self::Front | Self::Middle | Self::Back => 700.0,
            Self::Nozzle => 200.0,
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
}
