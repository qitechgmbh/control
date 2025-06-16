use super::MockMachine;
use control_core::actors::Actor;
use std::{
    future::Future,
    pin::Pin,
    time::{Duration, Instant},
};

/// Implements the `Actor` trait for the `MockMachine`.
///
/// # Parameters
/// - `_now_ts`: The current timestamp of type `Instant`.
///
/// # Returns
/// A pinned `Future` that resolves to `()` and is `Send`-safe. The future encapsulates the asynchronous behavior of the `act` method.
///
/// # Description
/// This method is called to perform periodic actions for the `MockMachine`. Specifically:
/// - It checks if the time elapsed since the last measurement emission exceeds 16 milliseconds.
/// - If the condition is met and the machine is in Running mode, it emits a sine wave data event.
/// - State events (frequency, mode) are only emitted when values change, not continuously.
///
/// The method ensures that the sine wave value is updated approximately 60 times per second (16ms intervals) when running.
///
impl Actor for MockMachine {
    fn act(&mut self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let now = Instant::now();

            // Emit initial state if this is the first call
            if self.last_emitted_frequency.is_none() || self.last_emitted_mode.is_none() {
                self.emit_sine_wave_state();
                self.emit_mode_state();
            }

            // Only emit sine wave if machine is in Running mode
            // The sine wave value is updated approximately 60 times per second
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 60.0)
            {
                self.emit_sine_wave();
                self.last_measurement_emit = now;
            }
        })
    }
}
