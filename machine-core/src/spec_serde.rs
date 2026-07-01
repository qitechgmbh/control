use serde::{Deserialize, Deserializer, de};

use crate::{MutationParameterType, PropertySampling, PropertySpec, spec::UomUnit};

#[derive(Deserialize)]
struct PropertyHelper {
    r#type: String,
    sampling: Option<PropertySampling>,
}

impl<'de> Deserialize<'de> for PropertySpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = PropertyHelper::deserialize(deserializer)?;

        let value = match helper.r#type.as_str() {
            "bool" => PropertySpec::Boolean,

            "int" => PropertySpec::Integer {
                sampling: helper
                    .sampling
                    .ok_or_else(|| de::Error::missing_field("PropertySampling"))?,
            },

            "float" => PropertySpec::Float {
                sampling: helper
                    .sampling
                    .ok_or_else(|| de::Error::missing_field("PropertySampling"))?,
            },

            "millimeter" => PropertySpec::UoM { 
                unit: UomUnit::Millimeter,
                sampling: helper
                    .sampling
                    .ok_or_else(|| de::Error::missing_field("PropertySampling"))?,
            },

            "meter" => PropertySpec::UoM { 
                unit: UomUnit::Meter,
                sampling: helper
                    .sampling
                    .ok_or_else(|| de::Error::missing_field("PropertySampling"))?,
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

#[cfg(test)]
mod test {
    use crate::MachineSpec;

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
        let v = yaml_serde::from_str::<MachineSpec>(X);

        println!("v: {v:?}");
    }
}
