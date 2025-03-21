pub mod analog_function_generator;
pub mod analog_input_getter;
pub mod digital_input_logger;
pub mod digital_output_setter;
pub mod digital_output_blinker;
pub mod digital_output_blinkers;
pub mod stepper_driver;
pub mod stepper_driver_max_speed;
pub mod stepper_driver_pulse_train;
pub mod temperature_input_logger;

use std::{future::Future, pin::Pin};

pub trait Actor: Send + Sync {
    fn act(&mut self, now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
