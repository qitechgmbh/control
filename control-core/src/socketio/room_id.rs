use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use crate::identification::MachineIdentificationUnique;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum RoomId {
    Main,
    Machine(MachineIdentificationUnique),
}

impl Serialize for RoomId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RoomId::Main => serializer.serialize_str("/main"),
            RoomId::Machine(id) => {
                let path = format!("/machine/{}/{}/{}", id.vendor, id.machine, id.serial);
                serializer.serialize_str(&path)
            }
        }
    }
}

impl<'de> Deserialize<'de> for RoomId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RoomIdVisitor;

        impl<'de> Visitor<'de> for RoomIdVisitor {
            type Value = RoomId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a room path")
            }

            fn visit_str<E>(self, value: &str) -> Result<RoomId, E>
            where
                E: de::Error,
            {
                if value == "/main" {
                    return Ok(RoomId::Main);
                }

                if let Some(machine_path) = value.strip_prefix("/machine/") {
                    let parts: Vec<&str> = machine_path.split('/').collect();
                    if parts.len() == 3 {
                        let vendor = parts[0]
                            .parse::<u16>()
                            .map_err(|_| E::custom("Invalid vendor id"))?;
                        let machine = parts[1]
                            .parse::<u16>()
                            .map_err(|_| E::custom("Invalid machine id"))?;
                        let serial = parts[2]
                            .parse::<u16>()
                            .map_err(|_| E::custom("Invalid serial id"))?;

                        return Ok(RoomId::Machine(MachineIdentificationUnique {
                            vendor,
                            machine,
                            serial,
                        }));
                    }
                }

                Err(E::custom(format!("Invalid room path: {}", value)))
            }
        }

        deserializer.deserialize_str(RoomIdVisitor)
    }
}

// Implement FromStr for convenience
impl FromStr for RoomId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "/main" {
            return Ok(RoomId::Main);
        }

        if let Some(machine_path) = s.strip_prefix("/machine/") {
            let parts: Vec<&str> = machine_path.split('/').collect();
            if parts.len() == 3 {
                let vendor = parts[0]
                    .parse::<u16>()
                    .map_err(|_| "Invalid vendor id".to_string())?;
                let machine = parts[1]
                    .parse::<u16>()
                    .map_err(|_| "Invalid machine id".to_string())?;
                let serial = parts[2]
                    .parse::<u16>()
                    .map_err(|_| "Invalid serial id".to_string())?;

                return Ok(RoomId::Machine(MachineIdentificationUnique {
                    vendor,
                    machine,
                    serial,
                }));
            }
        }

        Err(format!("Invalid room path: {}", s))
    }
}

// Implement Display for convenience
impl fmt::Display for RoomId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RoomId::Main => write!(f, "/main"),
            RoomId::Machine(id) => write!(f, "/machine/{}/{}/{}", id.vendor, id.machine, id.serial),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, to_string};
    use std::str::FromStr;

    #[test]
    fn test_serialize_main() {
        let room_id = RoomId::Main;
        let serialized = to_string(&room_id).unwrap();
        assert_eq!(serialized, "\"/main\"");
    }

    #[test]
    fn test_serialize_machine() {
        let machine_id = MachineIdentificationUnique {
            vendor: 123,
            machine: 456,
            serial: 789,
        };
        let room_id = RoomId::Machine(machine_id);
        let serialized = to_string(&room_id).unwrap();
        assert_eq!(serialized, "\"/machine/123/456/789\"");
    }

    #[test]
    fn test_deserialize_main() {
        let json = "\"/main\"";
        let room_id: RoomId = from_str(json).unwrap();
        assert!(matches!(room_id, RoomId::Main));
    }

    #[test]
    fn test_deserialize_machine() {
        let json = "\"/machine/123/456/789\"";
        let room_id: RoomId = from_str(json).unwrap();

        match room_id {
            RoomId::Machine(id) => {
                assert_eq!(id.vendor, 123);
                assert_eq!(id.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected RoomId::Machine"),
        }
    }

    #[test]
    fn test_deserialize_invalid_path() {
        let json = "\"/invalid/path\"";
        let result: Result<RoomId, _> = from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_invalid_machine_format() {
        let json = "\"/machine/not-a-number/456/789\"";
        let result: Result<RoomId, _> = from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_incomplete_machine_path() {
        let json = "\"/machine/123/456\"";
        let result: Result<RoomId, _> = from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_main() {
        let room_id = RoomId::from_str("/main").unwrap();
        assert!(matches!(room_id, RoomId::Main));
    }

    #[test]
    fn test_from_str_machine() {
        let room_id = RoomId::from_str("/machine/123/456/789").unwrap();

        match room_id {
            RoomId::Machine(id) => {
                assert_eq!(id.vendor, 123);
                assert_eq!(id.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected RoomId::Machine"),
        }
    }

    #[test]
    fn test_from_str_invalid() {
        let result = RoomId::from_str("/invalid/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_main() {
        let room_id = RoomId::Main;
        assert_eq!(format!("{}", room_id), "/main");
    }

    #[test]
    fn test_display_machine() {
        let machine_id = MachineIdentificationUnique {
            vendor: 123,
            machine: 456,
            serial: 789,
        };
        let room_id = RoomId::Machine(machine_id);
        assert_eq!(format!("{}", room_id), "/machine/123/456/789");
    }

    #[test]
    fn test_roundtrip_main() {
        let original = RoomId::Main;
        let serialized = to_string(&original).unwrap();
        let deserialized: RoomId = from_str(&serialized).unwrap();
        assert!(matches!(deserialized, RoomId::Main));
    }

    #[test]
    fn test_roundtrip_machine() {
        let machine_id = MachineIdentificationUnique {
            vendor: 123,
            machine: 456,
            serial: 789,
        };
        let original = RoomId::Machine(machine_id);
        let serialized = to_string(&original).unwrap();
        let deserialized: RoomId = from_str(&serialized).unwrap();

        match deserialized {
            RoomId::Machine(id) => {
                assert_eq!(id.vendor, 123);
                assert_eq!(id.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected RoomId::Machine"),
        }
    }

    #[test]
    fn test_string_roundtrip() {
        // Test using Display and FromStr
        let original = RoomId::Machine(MachineIdentificationUnique {
            vendor: 123,
            machine: 456,
            serial: 789,
        });

        let string_repr = format!("{}", original);
        let parsed = RoomId::from_str(&string_repr).unwrap();

        match parsed {
            RoomId::Machine(id) => {
                assert_eq!(id.vendor, 123);
                assert_eq!(id.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected RoomId::Machine"),
        }
    }
}
