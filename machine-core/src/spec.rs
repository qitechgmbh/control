use std::collections::HashMap;
use indexmap::IndexMap;
use crate::MachineIdentification;

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug)]
pub struct MachineSpec {
    pub name: String,
    pub identification: MachineIdentification,
    pub properties: HashMap<String, PropertySpecNode>,
    pub mutations: HashMap<String, IndexMap<String, MutationParameterType>>,
}

impl MachineSpec {
    pub fn find_property<'a>(
        &'a self,
        path: &str,
    ) -> Option<&'a PropertySpec> {
        let mut parts = path.split('.');

        let mut current = self.properties.get(parts.next()?)?;

        for part in parts {
            match current {
                PropertySpecNode::Group(map) => {
                    current = map.get(part)?;
                }
                PropertySpecNode::Property(_) => {
                    return None;
                }
            }
        }

        match current {
            PropertySpecNode::Property(p) => Some(p),
            PropertySpecNode::Group(_) => None,
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
#[derive(Debug)]
pub enum PropertySpecNode {
    Property(PropertySpec),
    Group(HashMap<String, PropertySpecNode>),
}

#[derive(Debug)]
pub enum PropertySpec {
    Boolean,
    String,
    Integer{sampling: PropertySampling},
    Float{sampling: PropertySampling},
    UoM {
        sampling: PropertySampling
    },
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Debug)]
pub enum PropertySampling {
    Simple,
    MinMax,
    Complete,
}

#[derive(Debug)]
pub enum MutationParameterType {
    Float{ min: Option<f64>, max: Option<f64> },
    Integer{ min: Option<i64>, max: Option<i64> },
    Boolean,
    String
}
