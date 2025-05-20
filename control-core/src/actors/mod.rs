pub mod analog_function_generator;
pub mod analog_input_getter;
pub mod digital_input_getter;
pub mod digital_input_logger;
pub mod digital_output_blinker;
pub mod digital_output_blinkers;
pub mod digital_output_setter;
pub mod mitsubishi_inverter_rs485;
pub mod stepper_driver_el70x1;
pub mod stepper_driver_max_speed;
pub mod stepper_driver_pulse_train;
pub mod temperature_input_logger;

use std::{pin::Pin, time::Instant};

pub trait Actor: Send + Sync {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
