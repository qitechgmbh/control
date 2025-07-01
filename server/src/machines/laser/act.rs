use super::LaserMachine;
use control_core::actors::Actor;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// Implements the `Actor` trait for the `LaserMachine`.
///
/// # Parameters
/// - `_now_ts`: The current timestamp of type `Instant`.
///
/// # Returns
/// A pinned `Future` that resolves to `()` and is `Send`-safe. The future encapsulates the asynchronous behavior of the `act` method.
///
/// # Description
/// This method is called to perform periodic actions for the `LaserMachine`. Specifically:
/// - It checks if the time elapsed since the last measurement emission exceeds 20 milliseconds.
/// - If the condition is met, it asynchronously emits Laser data by calling `emit_laser_data` and updates the `last_measurement_emit` timestamp.
///
/// The method ensures that the diameter value is updated approximately 60 times per second.
///
impl Actor for LaserMachine {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // The live values are updated approximately 60 times per second
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_diameter();
                self.emit_laser_state();
                self.last_measurement_emit = now;
            }
        })
    }
}
