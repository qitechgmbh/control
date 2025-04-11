use super::Actor;
use ethercat_hal::io::analog_output::AnalogOutput;
use std::{
    f64::consts::PI,
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};
use uom::si::{angle::radian, f64::Angle, ratio::ratio};
pub type AnalogFunction = Box<dyn Fn(Duration) -> f64 + Send + Sync>;

/// Can module analog output with a function
pub struct AnalogFunctionGenerator {
    output: AnalogOutput,
    function: AnalogFunction,
    start_ts: Instant,
}

impl AnalogFunctionGenerator {
    pub fn new(output: AnalogOutput, function: AnalogFunction) -> Self {
        Self {
            output,
            function,
            start_ts: Instant::now(),
        }
    }
}

impl Actor for AnalogFunctionGenerator {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let diff = now - self.start_ts;
            let value = (self.function)(diff);
            (self.output.write)((value as f32).into()).await;
        })
    }
}

pub fn analog_sine(amplitude: f64, normal: f64, period: Duration) -> AnalogFunction {
    Box::new(move |dt: Duration| {
        // Convert to seconds as f64 for calculation
        let dt_secs = dt.as_secs_f64();
        let period_secs = period.as_secs_f64();
        let phase_secs = dt_secs % period_secs;
        let angle = Angle::new::<radian>(2.0 * PI * phase_secs / period_secs);
        let value = angle.sin().get::<ratio>();
        value * (amplitude / 2.0) + normal + (amplitude / 2.0)
    })
}

pub fn analog_multiply<const N: usize>(functions: [AnalogFunction; N]) -> AnalogFunction {
    Box::new(move |dt: Duration| functions.iter().fold(1.0, |acc, func| acc * func(dt)))
}

pub fn analog_sawtooth(amplitude: f64, normal: f64, period: Duration) -> AnalogFunction {
    Box::new(move |dt: Duration| {
        let dt_secs = dt.as_secs_f64();
        let period_secs = period.as_secs_f64();
        let phase_secs = dt_secs % period_secs;
        let value = phase_secs / period_secs;
        value * amplitude + normal
    })
}

pub fn analog_square(amplitude: f64, normal: f64, period: Duration) -> AnalogFunction {
    Box::new(move |dt: Duration| {
        let dt_secs = dt.as_secs_f64();
        let period_secs = period.as_secs_f64();
        let phase_secs = dt_secs % period_secs;
        let value = if phase_secs < period_secs / 2.0 {
            1.0
        } else {
            -1.0
        };
        value * amplitude + normal
    })
}

pub fn analog_triangle(amplitude: f64, normal: f64, period: Duration) -> AnalogFunction {
    Box::new(move |dt: Duration| {
        let dt_secs = dt.as_secs_f64();
        let period_secs = period.as_secs_f64();
        let phase_secs = dt_secs % period_secs;
        let value = if phase_secs < period_secs / 2.0 {
            phase_secs / period_secs * 2.0
        } else {
            2.0 - phase_secs / period_secs * 2.0
        };
        value * amplitude + normal
    })
}
