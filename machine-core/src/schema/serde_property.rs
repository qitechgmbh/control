use std::collections::HashMap;

use serde::{Deserialize, Deserializer, de};

use crate::{
    PropertySchema, PropertySchemaNode, 
    schema::{PropertyLimits, PropertyRepresentation, PropertyTarget, UomUnit}
};

impl<'de> Deserialize<'de> for PropertySchemaNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = yaml_serde::Value::deserialize(deserializer)?;

        let property = PropertySchema::deserialize(value.clone());
        if let Ok(v) = property {
            return Ok(Self::Value(v));
        }

        let group = HashMap::<String, PropertySchemaNode>::deserialize(value.clone());
        if let Ok(v) = group {
            return Ok(Self::Group(v));
        }

        Err(D::Error::custom(format!(
            "did not match PropertySchemaNode\n\
             Property: {}\n\
             Group: {}",
            property.unwrap_err(),
            group.unwrap_err(),
        )))
    }
}

impl<'de> Deserialize<'de> for PropertySchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use PropertyTypeHelper::*;

        match PropertySchemaHelper::deserialize(deserializer)? {
            PropertySchemaHelper::Type(helper) => {
                let schema = match helper {
                    Bool => PropertySchema::Boolean,
                    Int => PropertySchema::Integer { 
                        repr: Default::default(), 
                        limits: Default::default(), 
                        target: Default::default(),
                    },
                    Float => PropertySchema::Float { 
                        repr: Default::default(), 
                        limits: Default::default(), 
                        target: Default::default(),
                    },
                    Meter => init_uom_default(UomUnit::Meter),
                    Millimeter => init_uom_default(UomUnit::Millimeter),
                };

                Ok(schema)
            },
            PropertySchemaHelper::Info(helper) => {
                match helper.r#type {
                    Bool => Ok(PropertySchema::Boolean),

                    Int => {
                        let repr = helper.repr.unwrap_or_default();
                        let limits = init_limits(helper.limits, Number::as_i64::<D::Error>)?;
                        let target = helper.target.unwrap_or_default();
                        Ok(PropertySchema::Integer { repr, limits, target })
                    }

                    Float => {
                        let repr = helper.repr.unwrap_or_default();
                        let limits = init_limits(helper.limits, Number::as_f64::<D::Error>)?;
                        let target = helper.target.unwrap_or_default();
                        Ok(PropertySchema::Float { repr, limits, target })
                    },

                    Millimeter => init_uom(helper, UomUnit::Millimeter),
                    Meter => init_uom(helper, UomUnit::Meter),
                }
            },
        }
    }
}

#[derive(Debug)]
enum PropertySchemaHelper {
    Type(PropertyTypeHelper),
    Info(PropertyInfoHelper),
}

impl<'de> Deserialize<'de> for PropertySchemaHelper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = yaml_serde::Value::deserialize(deserializer)?;

        let _type = PropertyTypeHelper::deserialize(value.clone());
        if let Ok(v) = _type {
            return Ok(Self::Type(v));
        }

        let info = PropertyInfoHelper::deserialize(value.clone());
        if let Ok(v) = info {
            return Ok(Self::Info(v));
        }

        Err(D::Error::custom(format!(
            "Failed to deserialize PropertySchema\n\
             Value: {}\n\
             Group: {}",
            _type.unwrap_err(),
            info.unwrap_err(),
        )))
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PropertyTypeHelper {
    Bool,
    Int,
    Float,
    Millimeter,
    Meter,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PropertyInfoHelper {
    r#type: PropertyTypeHelper,
    repr: Option<PropertyRepresentation>,
    target: Option<PropertyTarget>,
    limits: Option<PropertyLimitsHelper>,
}

#[derive(Debug, Deserialize)]
struct PropertyLimitsHelper {
    min: Option<Number>,
    max: Option<Number>,
}

fn init_uom_default(unit: UomUnit) -> PropertySchema {
    PropertySchema::UoM { 
        unit,
        repr: Default::default(),
        limits: Default::default(),
        target: Default::default(),
    }
}

fn init_limits<T, E>(
    helper: Option<PropertyLimitsHelper>,
    conv: impl Fn(Number) -> Result<T, E>,
) -> Result<PropertyLimits<T>, E> {
    match helper {
        Some(l) => Ok(PropertyLimits {
            min: l.min.map(&conv).transpose()?,
            max: l.max.map(conv).transpose()?,
        }),
        None => Ok(PropertyLimits {
            min: None,
            max: None,
        }),
    }
}

fn init_uom<E: de::Error>(helper: PropertyInfoHelper, unit: UomUnit) -> Result<PropertySchema, E> {
    let repr = helper.repr.unwrap_or_default();
    let limits = init_limits(helper.limits, Number::as_f64::<E>)?;
    let target = helper.target.unwrap_or_default();
    Ok(PropertySchema::UoM { unit, repr, limits, target })
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    fn as_i64<E: de::Error>(self) -> Result<i64, E> {
        match self {
            Number::Int(v) => Ok(v),
            Number::Float(_) => Err(E::custom("expected integer")),
        }
    }

    fn as_f64<E: de::Error>(self) -> Result<f64, E> {
        match self {
            Number::Float(v) => Ok(v),
            Number::Int(_) => Err(E::custom("expected float")),
        }
    }
}
