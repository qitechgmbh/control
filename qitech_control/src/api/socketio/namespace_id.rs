use std::fmt;
use std::str::FromStr;

use qitech_framework::MachineIdentification;
use qitech_framework::MachineIdentificationUnique;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Visitor;
use serde::de::{self};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum NamespaceId {
    Main,
    Machine(MachineIdentificationUnique),
}

impl ToString for NamespaceId {
    fn to_string(&self) -> String {
        match self {
            Self::Main => "/main".to_string(),
            Self::Machine(ident_unique) => {
                format!(
                    "/machine/{}/{}/{}",
                    ident_unique.identification.vendor_id,
                    ident_unique.identification.machine_id,
                    ident_unique.serial
                )
            }
        }
    }
}

impl Serialize for NamespaceId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
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
                        let vendor_id = parts[0]
                            .parse::<u16>()
                            .map_err(|_| E::custom("Invalid vendor id"))?;
                        let machine_id = parts[1]
                            .parse::<u16>()
                            .map_err(|_| E::custom("Invalid machine id"))?;
                        let serial = parts[2]
                            .parse::<u16>()
                            .map_err(|_| E::custom("Invalid serial id"))?;

                        return Ok(NamespaceId::Machine(MachineIdentificationUnique {
                            identification: MachineIdentification {
                                vendor_id,
                                machine_id,
                            },
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

impl FromStr for NamespaceId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "/main" {
            return Ok(Self::Main);
        }

        if let Some(machine_path) = s.strip_prefix("/machine/") {
            let parts: Vec<&str> = machine_path.split('/').collect();
            if parts.len() == 3 {
                let vendor_id = parts[0]
                    .parse::<u16>()
                    .map_err(|_| "Invalid vendor id".to_string())?;
                let machine_id = parts[1]
                    .parse::<u16>()
                    .map_err(|_| "Invalid machine id".to_string())?;
                let serial = parts[2]
                    .parse::<u16>()
                    .map_err(|_| "Invalid serial id".to_string())?;

                return Ok(Self::Machine(MachineIdentificationUnique {
                    identification: MachineIdentification {
                        vendor_id,
                        machine_id,
                    },
                    serial,
                }));
            }
        }

        Err(format!("Invalid namespace path: {}", s))
    }
}
