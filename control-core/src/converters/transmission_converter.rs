use uom::si::{angular_velocity::revolution_per_minute, f64::AngularVelocity};

#[derive(Debug, Clone)]
pub struct TransmissionConverter {
    pub transmission_ratio: f64,
}

impl TransmissionConverter {
    pub fn new() -> Self {
        Self {
            transmission_ratio: 34.0,
        }
    }

    pub fn calculate_screw_output_rpm(&self, rpm: AngularVelocity) -> AngularVelocity {
        rpm / self.transmission_ratio
    }

    pub fn calculate_screw_input_rpm(&self, screw_rpm: AngularVelocity) -> AngularVelocity {
        screw_rpm * self.transmission_ratio
    }
}
