use super::stop::{SafetyStop, StopReason};
use crate::gluetex::{Gluetex, GluetexMode, OperationMode};
use units::velocity::meter_per_second;

impl Gluetex {
    /// Apply a safety stop: shutdown motors/controllers, set Hold + Setup, optionally disable heaters.
    pub(crate) fn apply_safety_stop(&mut self, stop: SafetyStop) {
        let reason = stop.reason();
        let puller_speed_m_per_min = self
            .puller_speed_controller
            .last_speed
            .get::<meter_per_second>()
            * 60.0;

        if stop.disables_heaters() {
            tracing::warn!(
                ?reason,
                mode = ?self.mode,
                operation_mode = ?self.operation_mode,
                puller_speed_m_per_min,
                "safety stop — all motors and heaters disabled"
            );
        } else {
            tracing::warn!(
                ?reason,
                mode = ?self.mode,
                operation_mode = ?self.operation_mode,
                puller_speed_m_per_min,
                "safety stop — all motors disabled, heaters kept"
            );
        }

        self.operation_mode = OperationMode::Setup;
        self.set_mode(&GluetexMode::Hold);

        if stop.disables_heaters() {
            self.heaters.enabled = false;
            self.heaters.disallow_all_heating();
        }

        self.update_status_output();
        self.emit_safety_stop(stop);
        self.shutdown_motors();
    }

    /// Disable all motion hardware and speed controllers (shared by all stop profiles).
    pub(crate) fn shutdown_motors(&mut self) {
        self.spool.set_enabled(false);
        self.puller.set_enabled(false);
        self.slave_puller.set_enabled(false);
        self.traverse.set_enabled(false);
        self.stepper_3.set_enabled(false);
        self.stepper_4.set_enabled(false);
        self.stepper_5.set_enabled(false);

        self.spool_speed_controller.set_enabled(false);
        self.puller_speed_controller.set_enabled(false);
        self.slave_puller_speed_controller.set_enabled(false);

        self.stepper_3_controller.on_safety_stop();
        self.stepper_4_controller.on_safety_stop();
        self.stepper_5_controller.on_safety_stop();

        self.stepper_5_tension_controller.set_enabled(false);
        self.traverse_controller.set_enabled(false);
        self.valve_controller.set_enabled(false);
        let _ = self.valve.set(false);
    }

    pub(crate) fn safety_stop_motors_only(reason: StopReason) -> SafetyStop {
        SafetyStop::MotorsOnly { reason }
    }

    pub(crate) fn safety_stop_full(reason: StopReason) -> SafetyStop {
        SafetyStop::Full { reason }
    }
}
