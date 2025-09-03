use super::LaserMachine;
use control_core::machines::new::MachineAct;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// Implements the `MachineAct` trait for the `LaserMachine`.
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
/// - If the condition is met, it asynchronously emits live values at 60 FPS.
///
/// The method ensures that the diameter value is updated approximately 60 times per second.
///
impl MachineAct for LaserMachine {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            // more than 33ms have passed since last emit (30 "fps" target)
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0)
            {
                self.maybe_emit_state_event();
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
