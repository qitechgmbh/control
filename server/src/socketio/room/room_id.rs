use crate::ethercat::device_identification::MachineIdentificationUnique;
use serde::{Deserialize, Serialize};
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RoomId {
    Main,
    Machine(MachineIdentificationUnique),
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let representation = match self {
            RoomId::Main => RoomIdSerialized::Main(true),
            RoomId::Machine(machine_identification_unique) => {
                RoomIdSerialized::Machine(machine_identification_unique.clone())
            }
        };

        return representation.serialize(serializer);
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let representation = RoomIdSerialized::deserialize(deserializer)?;

        match representation {
            RoomIdSerialized::Main(_) => Ok(RoomId::Main),
            RoomIdSerialized::Machine(machine_identification_unique) => {
                Ok(RoomId::Machine(machine_identification_unique))
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
enum RoomIdSerialized {
    Main(bool),
    Machine(MachineIdentificationUnique),
}
