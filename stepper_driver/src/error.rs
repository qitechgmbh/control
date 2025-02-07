use core::fmt;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Error {
    RadiusNotSet,
    StepsPerRevolutionNotSet,
    ExceedsMaxSpeed,
    ExceedsLimits,
    StepperOutsideOfLimits,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::RadiusNotSet => write!(f, "Radius not set"),
            Error::StepsPerRevolutionNotSet => write!(f, "Steps per revolution not set"),
            Error::ExceedsMaxSpeed => write!(f, "Exceeds max speed"),
            Error::ExceedsLimits => write!(f, "Exceeds limits"),
            Error::StepperOutsideOfLimits => write!(f, "Stepper is currently outside of limits"),
        }
    }
}
