use serde::{Deserialize, Deserializer, de::{self, DeserializeOwned}};

use crate::{
    PropertySchema, 
    schema::{PropertyLimits, PropertyRepresentation, PropertyTarget, UomUnit}
};

#[derive(Deserialize)]
struct PropertyHelper {
    r#type: String,
    repr: Option<PropertyRepresentation>,
    target: Option<PropertyTarget>,
    limits: Option<PropertyLimitsHelper>,
}

#[derive(Deserialize)]
struct PropertyLimitsHelper {
    min: Option<yaml_serde::Value>,
    max: Option<yaml_serde::Value>,
}

impl PropertyLimitsHelper {
    pub fn to_limits<T>(&self) -> Result<PropertyLimits<T>, yaml_serde::Error>
    where
        T: Clone + DeserializeOwned,
    {
        let min = self.min.clone().map(T::deserialize);
        let max = self.max.clone().map(T::deserialize);

        let min = min.transpose()?;
        let max = max.transpose()?;

        Ok(PropertyLimits { min, max })
    }
}

impl<'de> Deserialize<'de> for PropertySchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = PropertyHelper::deserialize(deserializer)?;

        let value = match helper.r#type.as_str() {
            "bool" => PropertySchema::Boolean,

            "int" => {
                let repr = helper.repr.unwrap_or_default();

                let limits: PropertyLimits<i64> = match helper.limits {
                    Some(v) => v.to_limits()?,
                    None => PropertyLimits { min: None, max: None },
                };

                let target = helper.target.unwrap_or_default();
                PropertySchema::Integer { repr, limits, target }
            }

            "float" => PropertySchema::Float {
                repr: helper.repr.unwrap_or_default(),
                limits: PropertyLimits {
                    min: Some(0.0), // TODO: implement
                    max: Some(0.0),
                },
                target: helper.target.unwrap_or_default(),
            },

            "millimeter" => PropertySchema::UoM { 
                unit: UomUnit::Millimeter,
                repr: helper.repr.unwrap_or_default(),
                limits: PropertyLimits {
                    min: Some(0.0), // TODO: implement
                    max: Some(0.0),
                },
                target: helper.target.unwrap_or_default(),
            },

            "meter" => PropertySchema::UoM { 
                unit: UomUnit::Meter,
                repr: helper.repr.unwrap_or_default(),
                limits: PropertyLimits {
                    min: Some(0.0), // TODO: implement
                    max: Some(0.0),
                },
                target: helper.target.unwrap_or_default(),
            },

            other => {
                return Err(de::Error::unknown_variant(
                    other,
                    &[
                        "boolean",
                        "string",
                        "integer",
                        "float",
                        "millimeter",
                        "meter",
                    ],
                ))
            }
        };

        Ok(value)
    }
}

/*
impl<'de> Deserialize<'de> for MutationParameterType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use MutationParameterTypePure::*;

        let value = match MutationParameterHelper::deserialize(deserializer)? {
            MutationParameterHelper::Shorthand(ty) => {
                match ty {
                    Float => MutationParameterType::Float { min: None, max: None },
                    Integer => MutationParameterType::Integer { min: None, max: None },
                    Boolean => MutationParameterType::Boolean,
                    String => MutationParameterType::String,
                }
            },
            MutationParameterHelper::Detailed { ty, min, max } => {
                match ty {
                    Float => {
                        let min = if let Some(v) = min {
                            let Number::Float(x) = v else {
                                return Err(de::Error::custom(
                                    "min must be a float, received integer"
                                ));
                            };

                            Some(x)
                        } else {
                            None
                        };
                        
                        let max = if let Some(v) = max {
                            let Number::Float(x) = v else {
                                return Err(de::Error::custom(
                                    "max must be a float, received integer"
                                ));
                            };

                            Some(x)
                        } else {
                            None
                        };

                        MutationParameterType::Float  { min, max }
                    }
                    Integer => {
                        let min = if let Some(v) = min {
                            let Number::Int(x) = v else {
                                return Err(de::Error::custom(
                                    "min must be an integer, received float"
                                ));
                            };

                            Some(x)
                        } else {
                            None
                        };
                        
                        let max = if let Some(v) = max {
                            let Number::Int(x) = v else {
                                return Err(de::Error::custom(
                                    "max must be an integer, received float"
                                ));
                            };

                            Some(x)
                        } else {
                            None
                        };

                        MutationParameterType::Integer { min, max }
                    }
                    _ => {
                        if min.is_some() || max.is_some() {
                            return Err(de::Error::custom("Bounds are not supported for this type!"))
                        }

                        match ty {
                            Boolean => MutationParameterType::Boolean,
                            String => MutationParameterType::String,
                            _ => unreachable!(),
                        }
                    }
                }
            },
        };

        Ok(value)
    }
}


#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MutationParameterHelper {
    Shorthand(MutationParameterTypePure),

    Detailed {
        #[serde(rename = "type")]
        ty: MutationParameterTypePure,
        min: Option<Number>,
        max: Option<Number>,
    },
}

#[derive(Debug, Deserialize)]
pub enum MutationParameterTypePure {
    #[serde(rename = "float")]
    Float,
    #[serde(rename = "integer")]
    Integer,
    #[serde(rename = "bool")]
    Boolean,
    #[serde(rename = "string")]
    String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Number {
    Int(i64),
    Float(f64),
}

*/

#[cfg(test)]
mod test {
    use crate::MachineSchema;

    const X: &str = r#"
name: laser_v1

identification:
  vendor_id: 0
  machine_id: 0

properties:
  diameter:
    type: millimeter
    sampling: simple

  x_diameter:
    type: millimeter
    sampling: simple

  y_diameter: 
    type: millimeter
    sampling: simple

  input_tolerance:
    type: millimeter
    sampling: simple

  global_warning: 
    type: bool
    sampling: simple

  config: 
    target_diameter:
      type: millimeter
      sampling: simple

    higher_tolerance:
      type: millimeter
      sampling: simple

    lower_tolerance:
      type: millimeter
      sampling: simple

mutations:
  set_target_diameter: 
    value: 
      type: float
      min: 0.0
      max: 1.0

  set_lower_tolerance:
    value: float

  set_higher_tolerance: 
    value: float
"#;

    #[test]
    pub fn run() {
        let v = yaml_serde::from_str::<MachineSchema>(X);

        println!("v: {v:?}");
    }
}
