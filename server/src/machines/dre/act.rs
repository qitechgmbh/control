use super::DreMachine;
use control_core::actors::Actor;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// Implements the `Actor` trait for the `DreMachine`.
///
/// # Parameters
/// - `_now_ts`: The current timestamp of type `Instant`.
///
/// # Returns
/// A pinned `Future` that resolves to `()` and is `Send`-safe. The future encapsulates the asynchronous behavior of the `act` method.
///
/// # Description
/// This method is called to perform periodic actions for the `DreMachine`. Specifically:
/// - It checks if the time elapsed since the last measurement emission exceeds 20 milliseconds.
/// - If the condition is met, it asynchronously emits DRE data by calling `emit_dre_data` and updates the `last_measurement_emit` timestamp.
///
/// The method ensures that the diameter value is updated approximately 60 times per second.
///
impl Actor for DreMachine {
    fn act(&mut self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let now = Instant::now();
            // The diameter value is updated approximately 60 times per second
            if now.duration_since(self.last_measurement_emit) > Duration::from_millis(16) {
                self.emit_diameter();
                self.emit_dre_state();
                self.last_measurement_emit = now;
            }
        })
    }
}
