use super::Winder2;
use control_core::machines::new::MachineAct;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

impl MachineAct for Winder2 {
    fn act(&mut self, now: Instant) {
        static CYCLE_COUNTER: AtomicU64 = AtomicU64::new(0);
        let start = Instant::now();

        // original act code
        self.sync_spool_speed(now);
        self.sync_puller_speed(now);
        self.sync_traverse_speed();
        self.stop_or_pull_spool(now);

        if self.traverse_controller.did_change_state() {
            self.emit_state();
        }

        if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }

        // increment and check counter
        let cycle = CYCLE_COUNTER.fetch_add(1, Ordering::Relaxed);
        if cycle % 1000 == 0 {
            let duration = start.elapsed();
            tracing::info!(
                "[Winder2::act] Duration of act(): {} ns",
                duration.as_nanos()
            );
        }
    }
}
