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

*/
