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
    Millimeter{sampling: PropertySampling},
    Meter{sampling: PropertySampling},
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
