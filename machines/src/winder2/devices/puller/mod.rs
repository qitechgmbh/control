use std::time::Instant;

use units::ConstZero;
use units::acceleration::meter_per_minute_per_second;
use units::f64::*;
use units::jerk::meter_per_minute_per_second_squared;
use units::velocity::meter_per_minute;

use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;

use control_core::{
    controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    converters::linear_step_converter::LinearStepConverter,
};

mod types;

pub use types::GearRatio;
pub use types::RegulationMode;

pub struct Puller
{
    enabled:            bool,
    hardware_device:    StepperVelocityEL70x1,
    speed_regulators:   SpeedController,
    forward:            bool,
    gear_ratio:         GearRatio,

    acceleration_controller: LinearJerkSpeedController,
    // converter:  LinearStepConverter
    // last_speed: Velocity
}

impl Puller
{
    pub fn new(converter: LinearStepConverter) -> Self
    {
        let acceleration = Acceleration::new::<meter_per_minute_per_second>(5.0);
        let jerk         = Jerk::new::<meter_per_minute_per_second_squared>(10.0);
        let speed        = Velocity::new::<meter_per_minute>(50.0);

        Self {
            enabled: false,
        }
    }

    // setters


    // functions

    fn update(&mut self, t: Instant)
    {

    }


}

// pub fn sync_puller_speed(&mut self, t: Instant) {
//     let angular_velocity = self.puller_speed_controller.calc_angular_velocity(t);
//     let steps_per_second = self
//         .puller_speed_controller
//         .converter
//         .angular_velocity_to_steps(angular_velocity);
//     let _ = self.puller.set_speed(steps_per_second);
// }

pub struct SpeedController
{
    last_speed: Velocity,

    manual:   (),
    diameter: (),
}

pub struct DiameterSpeedRegulator
{
    current_diameter: Length,
    target_diameter:  Length,

    // pid controller
}

impl DiameterSpeedRegulator
{
    pub fn update(&mut self) -> Velocity
    {
        todo!()
    }

    pub fn set_current_diamater(&mut self, value: Length)
    {
        self.current_diameter = value;
    }

    pub fn set_target_diameter(&mut self, value: Length)
    {
        self.target_diameter = value;
    }
}