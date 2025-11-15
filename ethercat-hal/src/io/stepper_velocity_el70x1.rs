use std::{fmt, sync::Arc};

use crate::helpers::el70xx_velocity_converter::EL70x1VelocityConverter;
use anyhow::Error;
use smol::lock::RwLock;

/// Pulse Train Output (PTO) device
///
/// Generates digital puleses with a given frequency (not PWM) and counts them.
pub struct StepperVelocityEL70x1 {
    /// Write to the pulse train output
    set_output: Box<dyn Fn(StepperVelocityEL70x1Output) -> Result<(), Error> + Send + Sync>,
    /// Read the state of the pulse train output
    get_output: Box<dyn Fn() -> Result<StepperVelocityEL70x1Output, Error> + Send + Sync>,
    /// Read the state of the pulse train output
    get_input: Box<dyn Fn() -> Result<StepperVelocityEL70x1Input, Error> + Send + Sync>,
    /// Get the speed range configuration
    get_speed_range: Box<
        dyn Fn() -> Result<crate::shared_config::el70x1::EL70x1SpeedRange, Error> + Send + Sync,
    >,
}

impl fmt::Debug for StepperVelocityEL70x1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StepperVelocity")
    }
}

impl<'device> StepperVelocityEL70x1 {
    pub fn new<PORT, DEVICE>(device: Arc<RwLock<DEVICE>>, port: PORT) -> Self
    where
        PORT: Clone + Copy + Send + Sync + 'static,
        DEVICE: StepperVelocityEL70x1Device<PORT> + Send + Sync + 'static,
    {
        // build sync write closure
        let device1 = device.clone();
        let set_output = Box::new(
            move |value: StepperVelocityEL70x1Output| -> Result<(), Error> {
                smol::block_on(async {
                    let mut device = device1.write().await;
                    device.set_output(port, value)
                })
            },
        );

        // build sync get closure
        let device2 = device.clone();
        let get_input = Box::new(move || -> Result<StepperVelocityEL70x1Input, Error> {
            smol::block_on(async {
                let device = device2.read().await;
                device.get_input(port)
            })
        });

        let device3 = device.clone();
        let get_output = Box::new(move || -> Result<StepperVelocityEL70x1Output, Error> {
            smol::block_on(async {
                let device = device3.read().await;
                device.get_output(port)
            })
        });

        // build async get speed range closure
        let device3 = device;
        let get_speed_range = Box::new(
            move || -> Result<crate::shared_config::el70x1::EL70x1SpeedRange, Error> {
                smol::block_on(async {
                    let device = device3.read().await;
                    Ok(device.get_speed_range(port))
                })
            },
        );

        Self {
            set_output,
            get_input,
            get_output,
            get_speed_range,
        }
    }

    /// Set the speed in steps per second
    pub fn set_speed(&mut self, steps_per_second: f64) -> Result<(), Error> {
        // Get current state to preserve other output values
        let mut output = (self.get_output)().unwrap();

        // Get speed range from device to convert steps to velocity
        let speed_range = (self.get_speed_range)();
        let converter = EL70x1VelocityConverter::new(&speed_range.unwrap());
        let velocity = converter.steps_to_velocity(steps_per_second, true);

        output.velocity = velocity;

        // Write to device
        (self.set_output)(output)
    }

    /// Get the speed in steps per second
    pub fn get_speed(&self) -> i32 {
        let output = (self.get_output)().unwrap();

        let speed_range = (self.get_speed_range)();
        let converter = EL70x1VelocityConverter::new(&speed_range.unwrap());
        converter.velocity_to_steps(output.velocity, true) as i32
    }

    /// Enable or disable the stepper
    pub fn set_enabled(&mut self, enabled: bool) {
        // Get current state to preserve other output values
        let mut output = (self.get_output)().unwrap();

        output.enable = enabled;

        // Write to device
        (self.set_output)(output).unwrap();
    }

    /// Get the enabled state of the stepper
    pub fn is_enabled(&self) -> bool {
        let output = (self.get_output)().unwrap();
        output.enable
    }

    /// Get the current position of the stepper
    pub fn get_position(&self) -> i128 {
        let input = (self.get_input)().unwrap();
        input.counter_value
    }

    /// Set the position of the stepper
    pub fn set_position(&mut self, position: i128) {
        // Get current state to preserve other output values
        let mut output = (self.get_output)().unwrap();

        output.set_counter = Some(position);

        // Write to device
        (self.set_output)(output).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct StepperVelocityEL70x1Input {
    /// Combination of `counter_underflow`, `counter_overflow`, and `counter_value` from [`crate::pdo::el70x1::EncControlCompact`]
    pub counter_value: i128,

    /// `ready_to_enable` from [`crate::pdo::el70x1::StmStatus`]
    pub ready_to_enable: bool,

    /// `ready` from [`crate::pdo::el70x1::StmStatus`]
    pub ready: bool,

    /// `warning` from [`crate::pdo::el70x1::StmStatus`]
    pub warning: bool,

    /// `error` from [`crate::pdo::el70x1::StmStatus`]
    pub error: bool,

    /// `moving_positive` from [`crate::pdo::el70x1::StmStatus`]
    pub moving_positive: bool,

    /// `moving_negative` from [`crate::pdo::el70x1::StmStatus`]
    pub moving_negative: bool,

    /// `torque_reduced` from [`crate::pdo::el70x1::StmStatus`]
    pub torque_reduced: bool,
}

#[derive(Debug, Clone)]
pub struct StepperVelocityEL70x1Output {
    /// `velocity` from [`crate::pdo::el70x1::StmVelocity`]
    pub velocity: i16,

    /// `enable` from [`crate::pdo::el70x1::StmControl`]
    pub enable: bool,

    /// `reduce_torque` from [`crate::pdo::el70x1::StmControl`]
    pub reduce_torque: bool,

    /// `reset` from [`crate::pdo::el70x1::StmControl`]
    pub reset: bool,

    /// `set_counter` and `set_counter_value` from [`crate::pdo::el70x1::EncControl`]
    pub set_counter: Option<i128>,
}

pub trait StepperVelocityEL70x1Device<PORT>: Send + Sync
where
    PORT: Clone,
{
    fn set_output(&mut self, port: PORT, value: StepperVelocityEL70x1Output) -> Result<(), Error>;
    fn get_input(&self, port: PORT) -> Result<StepperVelocityEL70x1Input, Error>;
    fn get_output(&self, port: PORT) -> Result<StepperVelocityEL70x1Output, Error>;
    fn get_speed_range(&self, port: PORT) -> crate::shared_config::el70x1::EL70x1SpeedRange;
}
