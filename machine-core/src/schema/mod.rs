use std::collections::HashMap;
use serde::Deserialize;

use indexmap::IndexMap;
use crate::MachineIdentification;

mod serde_property;
mod serde_mutation;

#[derive(Debug, Deserialize)]
pub struct MachineSchema {
    pub name: String,
    pub identification: MachineIdentification,
    pub properties: HashMap<String, PropertySchemaNode>,

    #[serde(default)]
    pub mutations: HashMap<String, IndexMap<String, MutationParameterNode>>,

    #[serde(default)]
    pub descriptions: HashMap<String, PropertyDescriptionNode>
}

impl MachineSchema {
    // TODO: find property must account for repr
    pub fn find_property_schema(&self, path: &str) -> Option<&PropertySchema> {
        let mut parts = path.split('.');
        let mut current = self.properties.get(parts.next()?)?;

        for part in parts {
            match current {
                PropertySchemaNode::Group(map) => {
                    current = map.get(part)?;
                }
                PropertySchemaNode::Value(_) => {
                    return None;
                }
            }
        }

        match current {
            PropertySchemaNode::Value(p) => Some(p),
            PropertySchemaNode::Group(_) => None,
        }
    }

    pub fn find_property_description(&self, path: &str) -> Option<&HashMap<String, String>> {
        let mut parts = path.split('.');
        let mut current = self.descriptions.get(parts.next()?)?;

        for part in parts {
            match current {
                PropertyDescriptionNode::Group(map) => {
                    current = map.get(part)?;
                }
                PropertyDescriptionNode::Value(_) => {
                    return None;
                }
            }
        }

        match current {
            PropertyDescriptionNode::Value(p) => Some(p),
            PropertyDescriptionNode::Group(_) => None,
        }
    }
}

#[derive(Debug)]
pub enum PropertySchemaNode {
    Value(PropertySchema),
    Group(HashMap<String, PropertySchemaNode>),
}

#[derive(Debug)]
pub enum PropertySchema {
    Boolean,
    Integer {
        repr: PropertyRepresentation,
        limits: PropertyLimits<i64>,
        target: PropertyTarget,
    },
    Float {
        repr: PropertyRepresentation,
        limits: PropertyLimits<f64>,
        target: PropertyTarget,
    },
    UoM {
        unit: UomUnit,
        repr: PropertyRepresentation,
        limits: PropertyLimits<f64>,
        target: PropertyTarget,
    },
}

#[derive(Debug, Default)]
pub struct PropertyLimits<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyTarget {
    #[default]
    None,
    Exact,
    Range,
    Deviation,
}

#[derive(Debug)]
pub enum UomUnit {
    Millimeter,
    Meter,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyRepresentation {
    #[default]
    Latest,
    MinMax,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MutationParameterNode {
    Object(HashMap<String, MutationParameterNode>),
    List(Box<MutationParameterNode>),
    Enum(Vec<String>),
    Boolean,
    Float{ min: Option<f64>, max: Option<f64> },
    Integer{ min: Option<i64>, max: Option<i64> },
    String,
    // UoM { unit: UomUnit, min: Option<f64>, max: Option<f64> },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PropertyDescriptionNode {
    Value(HashMap<String, String>),
    Group(HashMap<String, PropertyDescriptionNode>),
}

#[cfg(test)]
mod test {
    use crate::MachineSchema;

    const DATA: &str = include_str!("../../../machine-schemas/laser_v1.yaml");

    #[test]
    pub fn run() {
        let v = yaml_serde::from_str::<MachineSchema>(DATA).unwrap();

        println!("v: {:?}", v.descriptions);

        panic!("oh no!");
    }
}
