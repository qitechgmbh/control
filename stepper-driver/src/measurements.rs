#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Positional {
    Steps(i128),
    Revolutions(f64),
    Degrees(f64),
    Meters(f64),
}

impl Positional {
    pub fn to_steps(
        &self,
        steps_per_revolution: Option<i128>,
        radius: Option<f64>,
    ) -> Result<i128, crate::error::Error> {
        match self {
            Positional::Steps(steps) => Ok(*steps),
            Positional::Revolutions(revolutions) => match steps_per_revolution {
                Some(steps_per_revolution) => {
                    Ok((revolutions * steps_per_revolution as f64) as i128)
                }
                None => Err(crate::error::Error::StepsPerRevolutionNotSet),
            },
            Positional::Degrees(degrees) => match steps_per_revolution {
                Some(steps_per_revolution) => {
                    Ok((degrees / 360.0 * steps_per_revolution as f64) as i128)
                }
                None => Err(crate::error::Error::StepsPerRevolutionNotSet),
            },
            Positional::Meters(meters) => match (steps_per_revolution, radius) {
                (Some(steps_per_revolution), Some(radius)) => {
                    Ok((meters / radius * steps_per_revolution as f64) as i128)
                }
                (None, _) => Err(crate::error::Error::StepsPerRevolutionNotSet),
                (_, None) => Err(crate::error::Error::RadiusNotSet),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Speed {
    StepsPerSeconds,
    RevolutionsPerSeconds,
    DegreesPerSeconds,
    MetersPerSeconds,
}

impl Speed {
    pub fn to_steps_per_seconds(
        &self,
        steps_per_revolution: Option<i128>,
        radius: Option<f64>,
    ) -> Result<f64, crate::error::Error> {
        match self {
            Speed::StepsPerSeconds => Ok(1.0),
            Speed::RevolutionsPerSeconds => match steps_per_revolution {
                Some(steps_per_revolution) => Ok(steps_per_revolution as f64),
                None => Err(crate::error::Error::StepsPerRevolutionNotSet),
            },
            Speed::DegreesPerSeconds => match steps_per_revolution {
                Some(steps_per_revolution) => Ok(steps_per_revolution as f64 / 360.0),
                None => Err(crate::error::Error::StepsPerRevolutionNotSet),
            },
            Speed::MetersPerSeconds => match (steps_per_revolution, radius) {
                (Some(steps_per_revolution), Some(radius)) => {
                    Ok(steps_per_revolution as f64 / radius)
                }
                (None, _) => Err(crate::error::Error::StepsPerRevolutionNotSet),
                (_, None) => Err(crate::error::Error::RadiusNotSet),
            },
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Acceleration {
    StepsPerSecondsSquared,
    RevolutionsPerSecondsSquared,
    DegreesPerSecondsSquared,
    MetersPerSecondsSquared,
}

impl Acceleration {
    pub fn to_steps_per_seconds_squared(
        &self,
        steps_per_revolution: Option<i128>,
        radius: Option<f64>,
    ) -> Result<f64, crate::error::Error> {
        match self {
            Acceleration::StepsPerSecondsSquared => Ok(1.0),
            Acceleration::RevolutionsPerSecondsSquared => match steps_per_revolution {
                Some(steps_per_revolution) => Ok(steps_per_revolution as f64),
                None => Err(crate::error::Error::StepsPerRevolutionNotSet),
            },
            Acceleration::DegreesPerSecondsSquared => match steps_per_revolution {
                Some(steps_per_revolution) => Ok(steps_per_revolution as f64 / 360.0),
                None => Err(crate::error::Error::StepsPerRevolutionNotSet),
            },
            Acceleration::MetersPerSecondsSquared => match (steps_per_revolution, radius) {
                (Some(steps_per_revolution), Some(radius)) => {
                    Ok(steps_per_revolution as f64 / radius)
                }
                (None, _) => Err(crate::error::Error::StepsPerRevolutionNotSet),
                (_, None) => Err(crate::error::Error::RadiusNotSet),
            },
        }
    }
}
