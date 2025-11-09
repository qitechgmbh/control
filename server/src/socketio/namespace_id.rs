use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

use machines::machine_identification::{MachineIdentification, MachineIdentificationUnique};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NamespaceId {
    Main,
    Machine(MachineIdentificationUnique),
}

impl Serialize for NamespaceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Main => serializer.serialize_str("/main"),
            Self::Machine(id) => {
                let path = format!(
                    "/machine/{}/{}/{}",
                    id.machine_identification.vendor, id.machine_identification.machine, id.serial
                );
                serializer.serialize_str(&path)
            }
        }
    }
}

impl<'de> Deserialize<'de> for NamespaceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NamespaceIdVisitor;

        impl Visitor<'_> for NamespaceIdVisitor {
            type Value = NamespaceId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a namespace path")
            }

            fn visit_str<E>(self, value: &str) -> Result<NamespaceId, E>
            where
                E: de::Error,
            {
                if value == "/main" {
                    return Ok(NamespaceId::Main);
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

                        return Ok(NamespaceId::Machine(MachineIdentificationUnique {
                            machine_identification: MachineIdentification { vendor, machine },
                            serial,
                        }));
                    }
                }

                Err(E::custom(format!("Invalid namespace path: {}", value)))
            }
        }

        deserializer.deserialize_str(NamespaceIdVisitor)
    }
}

// Implement FromStr for convenience
impl FromStr for NamespaceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "/main" {
            return Ok(Self::Main);
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

                return Ok(Self::Machine(MachineIdentificationUnique {
                    machine_identification: MachineIdentification { vendor, machine },
                    serial,
                }));
            }
        }

        Err(format!("Invalid namespace path: {}", s))
    }
}

// Implement Display for convenience
impl fmt::Display for NamespaceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Main => write!(f, "/main"),
            Self::Machine(id) => {
                write!(
                    f,
                    "/machine/{}/{}/{}",
                    id.machine_identification.vendor, id.machine_identification.machine, id.serial
                )
            }
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
        let namespace_id = NamespaceId::Main;
        let serialized = to_string(&namespace_id).unwrap();
        assert_eq!(serialized, "\"/main\"");
    }

    #[test]
    fn test_serialize_machine() {
        let machine_id = MachineIdentificationUnique {
            machine_identification: MachineIdentification {
                vendor: 123,
                machine: 456,
            },
            serial: 789,
        };
        let namespace_id = NamespaceId::Machine(machine_id);
        let serialized = to_string(&namespace_id).unwrap();
        assert_eq!(serialized, "\"/machine/123/456/789\"");
    }

    #[test]
    fn test_deserialize_main() {
        let json = "\"/main\"";
        let namespace_id: NamespaceId = from_str(json).unwrap();
        assert!(matches!(namespace_id, NamespaceId::Main));
    }

    #[test]
    fn test_deserialize_machine() {
        let json = "\"/machine/123/456/789\"";
        let namespace_id: NamespaceId = from_str(json).unwrap();

        match namespace_id {
            NamespaceId::Machine(id) => {
                assert_eq!(id.machine_identification.vendor, 123);
                assert_eq!(id.machine_identification.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected NamespaceId::Machine"),
        }
    }

    #[test]
    fn test_deserialize_invalid_path() {
        let json = "\"/invalid/path\"";
        let result: Result<NamespaceId, _> = from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_invalid_machine_format() {
        let json = "\"/machine/not-a-number/456/789\"";
        let result: Result<NamespaceId, _> = from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_incomplete_machine_path() {
        let json = "\"/machine/123/456\"";
        let result: Result<NamespaceId, _> = from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_main() {
        let namespace_id = NamespaceId::from_str("/main").unwrap();
        assert!(matches!(namespace_id, NamespaceId::Main));
    }

    #[test]
    fn test_from_str_machine() {
        let namespace_id = NamespaceId::from_str("/machine/123/456/789").unwrap();

        match namespace_id {
            NamespaceId::Machine(id) => {
                assert_eq!(id.machine_identification.vendor, 123);
                assert_eq!(id.machine_identification.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected NamespaceId::Machine"),
        }
    }

    #[test]
    fn test_from_str_invalid() {
        let result = NamespaceId::from_str("/invalid/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_display_main() {
        let namespace_id = NamespaceId::Main;
        assert_eq!(format!("{}", namespace_id), "/main");
    }

    #[test]
    fn test_display_machine() {
        let machine_id = MachineIdentificationUnique {
            machine_identification: MachineIdentification {
                vendor: 123,
                machine: 456,
            },
            serial: 789,
        };
        let namespace_id = NamespaceId::Machine(machine_id);
        assert_eq!(format!("{}", namespace_id), "/machine/123/456/789");
    }

    #[test]
    fn test_roundtrip_main() {
        let original = NamespaceId::Main;
        let serialized = to_string(&original).unwrap();
        let deserialized: NamespaceId = from_str(&serialized).unwrap();
        assert!(matches!(deserialized, NamespaceId::Main));
    }

    #[test]
    fn test_roundtrip_machine() {
        let machine_id = MachineIdentificationUnique {
            machine_identification: MachineIdentification {
                vendor: 123,
                machine: 456,
            },
            serial: 789,
        };
        let original = NamespaceId::Machine(machine_id);
        let serialized = to_string(&original).unwrap();
        let deserialized: NamespaceId = from_str(&serialized).unwrap();

        match deserialized {
            NamespaceId::Machine(id) => {
                assert_eq!(id.machine_identification.vendor, 123);
                assert_eq!(id.machine_identification.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected NamespaceId::Machine"),
        }
    }

    #[test]
    fn test_string_roundtrip() {
        // Test using Display and FromStr
        let original = NamespaceId::Machine(MachineIdentificationUnique {
            machine_identification: MachineIdentification {
                vendor: 123,
                machine: 456,
            },
            serial: 789,
        });

        let string_repr = format!("{}", original);
        let parsed = NamespaceId::from_str(&string_repr).unwrap();

        match parsed {
            NamespaceId::Machine(id) => {
                assert_eq!(id.machine_identification.vendor, 123);
                assert_eq!(id.machine_identification.machine, 456);
                assert_eq!(id.serial, 789);
            }
            _ => panic!("Expected NamespaceId::Machine"),
        }
    }
}
