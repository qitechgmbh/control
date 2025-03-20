use crate::ethercat::device_identification::MachineIdentificationUnique;
use serde::{Deserialize, Serialize};
use socketioxide_core::adapter::RoomParam;
use std::{borrow::Cow, hash::Hash};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RoomId {
    // `main`
    Main,
    // `machine-1-1-1`
    Machine(MachineIdentificationUnique),
}

// Implement From<String> for RoomId
impl From<String> for RoomId {
    fn from(s: String) -> Self {
        match s.as_str() {
            "main" => RoomId::Main,
            s if s.starts_with("machine-") => {
                let split = s.split('-').collect::<Vec<&str>>();
                if split.len() >= 4 {
                    if let (Ok(vendor), Ok(machine), Ok(serial)) = (
                        split[1].parse::<u32>(),
                        split[2].parse::<u32>(),
                        split[3].parse::<u32>(),
                    ) {
                        return RoomId::Machine(MachineIdentificationUnique {
                            vendor,
                            machine,
                            serial,
                        });
                    }
                }
                // Return Main as fallback for invalid machine IDs
                RoomId::Main
            }
            _ => RoomId::Main, // Default case
        }
    }
}

// Keep From<&str> for convenience
impl From<&str> for RoomId {
    fn from(s: &str) -> Self {
        String::from(s).into()
    }
}

impl From<Cow<'_, str>> for RoomId {
    fn from(s: Cow<'_, str>) -> Self {
        String::from(s).into()
    }
}

impl From<RoomId> for String {
    fn from(room_id: RoomId) -> Self {
        match room_id {
            RoomId::Main => String::from("main"),
            RoomId::Machine(machine_identification_unique) => {
                let vendor = machine_identification_unique.vendor;
                let machine = machine_identification_unique.machine;
                let serial = machine_identification_unique.serial;
                format!("machine-{}-{}-{}", vendor, machine, serial)
            }
        }
    }
}

impl From<RoomId> for Cow<'static, str> {
    fn from(room_id: RoomId) -> Self {
        String::from(room_id).into()
    }
}

impl RoomParam for RoomId {
    type IntoIter = std::iter::Once<Cow<'static, str>>;

    #[inline(always)]
    fn into_room_iter(self) -> Self::IntoIter {
        let room_string = match self {
            RoomId::Main => self.into(),
            RoomId::Machine(_) => self.into(),
        };

        std::iter::once(Cow::Owned(room_string))
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize as string first
        let string_value = String::deserialize(deserializer)?;
        // Then convert from &str to RoomId
        Ok(RoomId::from(string_value.as_str()))
    }
}

//Serialize
impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::from(self.clone()).serialize(serializer)
    }
}

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}
