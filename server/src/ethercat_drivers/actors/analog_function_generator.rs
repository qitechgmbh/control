use crate::ethercat_drivers::{
    actor::Actor, io::analog_output::AnalogOutput, utils::traits::ArcRwLock,
};
use ethercrab::std::ethercat_now;
use std::f32::consts::PI;
use std::{future::Future, pin::Pin};
use uom::si::{angle::radian, f32::Angle, ratio::ratio};

pub type AnalogFunction = Box<dyn Fn(u64) -> f32 + Send + Sync>;

/// Can module analog output with a function
pub struct AnalogFunctionGenerator {
    output: AnalogOutput,
    function: AnalogFunction,
    offset_ts: u64,
}

impl AnalogFunctionGenerator {
    pub fn new(output: AnalogOutput, function: AnalogFunction) -> Self {
        Self {
            output,
            function,
            offset_ts: ethercat_now(),
        }
    }
}

impl Actor for AnalogFunctionGenerator {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.output.state)().await;
            let diff_ns = state.output_ts - self.offset_ts;
            let value = (self.function)(diff_ns);
            (self.output.write)(value as f32).await;
        })
    }
}

impl ArcRwLock for AnalogFunctionGenerator {}

pub fn analog_sine(amplitude: f32, normal: f32, period_ns: u64) -> AnalogFunction {
    Box::new(move |time_ns: u64| {
        let phase = time_ns % period_ns;
        let angle = Angle::new::<radian>(2.0 * PI * (phase as f32) / (period_ns as f32));
        let value = angle.sin().get::<ratio>();
        value * (amplitude / 2.0) + normal + (amplitude / 2.0)
    })
}

pub fn analog_multiply<const N: usize>(functions: [AnalogFunction; N]) -> AnalogFunction {
    Box::new(move |time_ns: u64| functions.iter().fold(1.0, |acc, func| acc * func(time_ns)))
}

pub fn analog_sawtooth(amplitude: f32, normal: f32, period_ns: u64) -> AnalogFunction {
    Box::new(move |time_ns: u64| {
        let phase = time_ns % period_ns;
        let value = (phase as f32) / (period_ns as f32);
        value * amplitude + normal
    })
}

pub fn analog_square(amplitude: f32, normal: f32, period_ns: u64) -> AnalogFunction {
    Box::new(move |time_ns: u64| {
        let phase = time_ns % period_ns;
        let value = if phase < period_ns / 2 { 1.0 } else { -1.0 };
        value * amplitude + normal
    })
}

pub fn analog_triangle(amplitude: f32, normal: f32, period_ns: u64) -> AnalogFunction {
    Box::new(move |time_ns: u64| {
        let phase = time_ns % period_ns;
        let value = if phase < period_ns / 2 {
            (phase as f32) / (period_ns as f32) * 2.0
        } else {
            2.0 - (phase as f32) / (period_ns as f32) * 2.0
        };
        value * amplitude + normal
    })
}
