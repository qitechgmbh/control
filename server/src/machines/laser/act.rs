use super::LaserMachine;
use control_core::machines::new::MachineAct;
use std::time::{Duration, Instant};

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
use std::sync::atomic::{AtomicU64, Ordering};

impl MachineAct for LaserMachine {
    fn act(&mut self, now: Instant) {
        // Start high-resolution timer
        let start = Instant::now();

        self.update();

        // emit live values every ~33ms
        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }

        // End timer
        let elapsed = start.elapsed();
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let count = COUNTER.fetch_add(1, Ordering::Relaxed);

        // Log every 10th call to reduce spam
        if count % 1000 == 0 {
            tracing::info!(
                "[LaserMachine::act] Duration of act(): {} ns",
                elapsed.as_nanos()
            );
        }
    }
}
