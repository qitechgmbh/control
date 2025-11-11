use units::{
    angle::radian,
    angular_velocity::radian_per_second,
    f64::{Angle, AngularVelocity, Length, Velocity},
    length::meter,
    velocity::meter_per_second,
};

pub mod fixed;

/// A trait representing a mechanical transmission system that converts input to output
/// with a specific gear ratio.
///
/// This trait provides methods for working with transmission ratios and converting
/// between input and output values based on the transmission's gear ratio.
pub trait Transmission {
    /// Returns the transmission ratio of this transmission system.
    ///
    /// The ratio represents the relationship between input and output speeds/torques.
    /// A ratio greater than 1.0 indicates speed reduction (torque multiplication),
    /// while a ratio less than 1.0 indicates speed increase (torque reduction).
    ///
    /// # Returns
    ///
    /// The transmission ratio as a floating-point number.
    fn get_ratio(&self) -> f64;

    /// Converts an input value to the corresponding output value using the transmission ratio.
    ///
    /// This method multiplies the input by the transmission ratio to calculate the output.
    /// Typically used for converting input speed to output speed in a transmission system.
    ///
    /// # Arguments
    ///
    /// * `input` - The input value to be converted
    ///
    /// # Returns
    ///
    /// The output value after applying the transmission ratio.
    fn calculate_output(&self, input: f64) -> f64 {
        input * self.get_ratio()
    }

    /// Converts an output value to the corresponding input value using the transmission ratio.
    ///
    /// This method divides the output by the transmission ratio to calculate the required input.
    /// Typically used for determining what input is needed to achieve a desired output.
    ///
    /// # Arguments
    ///
    /// * `output` - The output value to be converted back to input
    ///
    /// # Returns
    ///
    /// The input value required to produce the given output.
    fn calculate_input(&self, output: f64) -> f64 {
        output / self.get_ratio()
    }

    fn calculate_angular_output(&self, input: Angle) -> Angle {
        Angle::new::<radian>(self.calculate_output(input.get::<radian>()))
    }

    fn calculate_angular_input(&self, output: Angle) -> Angle {
        Angle::new::<radian>(self.calculate_input(output.get::<radian>()))
    }

    fn calculate_angular_velocity_output(&self, input: AngularVelocity) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(
            self.calculate_output(input.get::<radian_per_second>()),
        )
    }

    fn calculate_angular_velocity_input(&self, output: AngularVelocity) -> AngularVelocity {
        AngularVelocity::new::<radian_per_second>(
            self.calculate_input(output.get::<radian_per_second>()),
        )
    }

    fn calculate_linear_output(&self, input: Length) -> Length {
        Length::new::<meter>(self.calculate_output(input.get::<meter>()))
    }

    fn calculate_linear_input(&self, output: Length) -> Length {
        Length::new::<meter>(self.calculate_input(output.get::<meter>()))
    }

    fn calculate_linear_velocity_output(&self, input: Velocity) -> Velocity {
        Velocity::new::<meter_per_second>(self.calculate_output(input.get::<meter_per_second>()))
    }

    fn calculate_linear_velocity_input(&self, output: Velocity) -> Velocity {
        Velocity::new::<meter_per_second>(self.calculate_input(output.get::<meter_per_second>()))
    }
}
