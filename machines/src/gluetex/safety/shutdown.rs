use super::inbox::PushOutcome;
use super::stop::{SafetyStop, StopReason};
use crate::gluetex::{Gluetex, GluetexMode, OperationMode};
use std::time::Instant;
use units::velocity::meter_per_second;

impl Gluetex {
    /// Sole entry point coordinator functions call when they detect a
    /// rising-edge safety condition. Records it into the pending inbox
    /// (deduped by reason kind while already pending), reconciles the
    /// aggregate mode/motor/heater side effects, and emits state.
    pub(crate) fn record_safety_message(&mut self, stop: SafetyStop) {
        let reason = stop.reason();
        let now = Instant::now();

        match self.pending_safety.push(stop, now) {
            PushOutcome::New(id) => {
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
                        "safety message raised — all motors and heaters disabled"
                    );
                } else {
                    tracing::warn!(
                        ?reason,
                        mode = ?self.mode,
                        operation_mode = ?self.operation_mode,
                        puller_speed_m_per_min,
                        "safety message raised — all motors disabled, heaters kept"
                    );
                }

                self.emit_safety_message_raised(id, stop);
            }
            PushOutcome::Updated(id) => {
                tracing::debug!(?reason, id, "safety message re-observed while pending");
            }
        }

        self.reconcile_safety_stops();
        self.emit_state();
    }

    /// Derives the aggregate mode/motor/heater side effects from the current
    /// pending-message list. Idempotent — safe to call every tick from every
    /// safety check.
    pub(crate) fn reconcile_safety_stops(&mut self) {
        let should_be_engaged = !self.pending_safety.is_empty();

        if should_be_engaged && !self.safety_engaged {
            self.safety_engaged = true;
            tracing::warn!(pending = self.pending_safety.len(), "safety stop engaged");
            self.operation_mode = OperationMode::Setup;
            self.set_mode(&GluetexMode::Hold);
            self.update_status_output();
            self.shutdown_motors();
        } else if !should_be_engaged && self.safety_engaged {
            self.safety_engaged = false;
            tracing::info!("safety stop cleared — all pending messages acknowledged");
            // Deliberately no auto-return to Production and no auto
            // re-enabling of heaters — both stay separate, explicit operator
            // actions (SetOperationMode / SetMode / SetHeatingEnabled).
        }

        if self.pending_safety.any_full_severity() && self.heaters.enabled {
            self.heaters.enabled = false;
            self.heaters.disallow_all_heating();
        }
    }

    /// Acknowledge (remove) a single pending safety message and relatch its
    /// underlying monitor so a still-present hazard is re-detected as a new
    /// message rather than silently suppressed.
    pub(crate) fn acknowledge_safety_message(&mut self, id: u64) {
        if let Some(msg) = self.pending_safety.acknowledge(id) {
            self.relatch_monitor_for(msg.reason);
        }
        self.reconcile_safety_stops();
        self.emit_state();
    }

    /// Acknowledge (remove) every pending safety message.
    pub(crate) fn acknowledge_all_safety_messages(&mut self) {
        let acknowledged = self.pending_safety.acknowledge_all();
        for msg in acknowledged {
            self.relatch_monitor_for(msg.reason);
        }
        self.reconcile_safety_stops();
        self.emit_state();
    }

    /// Reset the underlying monitor's latch/debounce state for `reason` so
    /// it re-arms and will re-detect the hazard on its very next real check
    /// if still physically present — producing a *new* pending message
    /// rather than allowing the machine to resume in a still-hazardous
    /// state.
    fn relatch_monitor_for(&mut self, reason: StopReason) {
        match reason {
            StopReason::WinderTensionArm
            | StopReason::TapeFeederTensionArm
            | StopReason::InletTensionArm
            | StopReason::Optris1Voltage
            | StopReason::Optris2Voltage
            | StopReason::Bandueberwachung => {
                // No action needed: these monitors already force-clear their
                // own `triggered`/debounce state the instant operation_mode
                // leaves Production, and reconcile_safety_stops already
                // forced Setup mode when this message was first raised. The
                // monitor is already relatched and will re-arm for real
                // (with its normal 200ms debounce) the moment the operator
                // next requests Production.
            }
            StopReason::SleepTimer => self.sleep_timer.reset(),
            StopReason::HeaterOverTemperature { .. } => {
                // No separate latch to reset: `run_heater_overtemperature_check`
                // gates purely on whether a HeaterOverTemperature message is
                // still present in `pending_safety`. Acknowledging just
                // removed it, so the very next act() tick re-evaluates
                // `any_over_temperature()` fresh and immediately raises a
                // brand-new message if still over max temperature. Heaters
                // themselves stay disabled regardless (see
                // reconcile_safety_stops) until the operator explicitly
                // re-enables them, so nothing re-heats as a side effect of
                // acknowledgement.
            }
        }
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

        // NOTE: stepper_5_tension_controller is intentionally NOT disabled here.
        // It is an operator toggle (tape-feeder follows the tension arm); the motor
        // itself is already cut above via stepper_5.set_enabled(false), so no tape
        // moves during the stop. Leaving the flag untouched preserves the operator's
        // setting so the coupling resumes as configured when Wind re-enables the motor.
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
