use std::collections::HashMap;
use serde::Deserialize;

// use indexmap::IndexMap;
use crate::MachineIdentification;

mod serde_impl;

#[derive(Debug, Deserialize)]
pub struct MachineSchema {
    pub name: String,
    pub identification: MachineIdentification,
    pub properties: HashMap<String, PropertySchemaNode>,
    // pub mutations: HashMap<String, IndexMap<String, MutationParameterType>>,
    pub descriptions: HashMap<String, PropertyDescriptionNode>
}

impl MachineSchema {
    // TODO: find property must account for repr
    pub fn find_property<'a>(
        &'a self,
        path: &str,
    ) -> Option<&'a PropertySchema> {
        let mut parts = path.split('.');

        let mut current = self.properties.get(parts.next()?)?;

        for part in parts {
            match current {
                PropertySchemaNode::Group(map) => {
                    current = map.get(part)?;
                }
                PropertySchemaNode::Property(_) => {
                    return None;
                }
            }
        }

        match current {
            PropertySchemaNode::Property(p) => Some(p),
            PropertySchemaNode::Group(_) => None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PropertySchemaNode {
    Property(PropertySchema),
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct PropertyLimits<T> {
    pub min: Option<T>,
    pub max: Option<T>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PropertyTarget {
    #[default]
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

/*
#[derive(Debug)]
pub enum MutationParameterType {
    Float{ min: Option<f64>, max: Option<f64> },
    Integer{ min: Option<i64>, max: Option<i64> },
    Boolean,
    String
}
*/

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PropertyDescriptionNode {
    Value(HashMap<String, String>),
    Group(HashMap<String, PropertyDescriptionNode>),
}
