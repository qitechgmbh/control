use super::Gluetex;
use crate::{MachineAct, MachineMessage, MachineValues};
use std::time::{Duration, Instant};

const LIVE_VALUES_EMIT_INTERVAL: Duration = Duration::from_micros(33_333);
const STATE_EMIT_INTERVAL: Duration = Duration::from_millis(500);

fn should_emit_state_periodically(
    now: Instant,
    last_state_emit: Instant,
    autotuning_active: bool,
    sleep_timer_enabled: bool,
) -> bool {
    let should_emit_state = autotuning_active || sleep_timer_enabled;
    should_emit_state && now.duration_since(last_state_emit) > STATE_EMIT_INTERVAL
}

fn should_emit_live_values(now: Instant, last_measurement_emit: Instant) -> bool {
    now.duration_since(last_measurement_emit) > LIVE_VALUES_EMIT_INTERVAL
}

impl MachineAct for Gluetex {
    fn act(&mut self, now: Instant) {
        let machine_message = self.api_receiver.try_recv();
        match machine_message {
            Ok(machine_message) => self.act_machine_message(machine_message),
            Err(_e) => (),
        };
        // sync the spool speed
        self.sync_spool_speed(now);

        // sync the puller speed
        self.sync_puller_speed(now);

        // sync the slave puller speed
        self.sync_slave_puller_speed(now);

        // record optris voltage readings with distance tracking for delayed readings
        self.record_optris_voltages(now);

        // sync valve
        self.sync_valve(now);

        // sync addon motor speeds
        self.sync_addon_motor_3_speed(now);
        self.sync_addon_motor_4_speed(now);
        self.sync_addon_motor_5_speed(now);

        // sync the traverse speed
        self.sync_traverse_speed();

        // automatically stops or pulls after N Meters if enabled
        self.stop_or_pull_spool(now);

        // check tension arm positions and trigger emergency stop if needed
        self.check_tension_arm_monitor();

        // check voltage monitors and trigger emergency stop if needed
        self.check_voltage_monitors();

        // check sleep timer and enter standby if inactive for too long
        self.check_sleep_timer(now);

        // update all temperature controllers
        self.temperature_controller_1.update(now);
        self.temperature_controller_2.update(now);
        self.temperature_controller_3.update(now);
        self.temperature_controller_4.update(now);
        self.temperature_controller_5.update(now);
        self.temperature_controller_6.update(now);

        // update status output based on current activity
        self.update_status_output();

        // Check for completed auto-tuning and emit results
        self.check_autotuning_results();

        if self.traverse_controller.did_change_state() {
            self.emit_state();
            self.last_state_emit = now;
        }

        let autotuning_active = self.temperature_controller_1.is_autotuning()
            || self.temperature_controller_2.is_autotuning()
            || self.temperature_controller_3.is_autotuning()
            || self.temperature_controller_4.is_autotuning()
            || self.temperature_controller_5.is_autotuning()
            || self.temperature_controller_6.is_autotuning();

        if should_emit_state_periodically(
            now,
            self.last_state_emit,
            autotuning_active,
            self.sleep_timer.config.enabled,
        ) {
            self.emit_state();
            self.last_state_emit = now;
        }

        // more than 33ms have passed since last emit (30 "fps" target)
        if should_emit_live_values(now, self.last_measurement_emit) {
            self.emit_live_values();
            self.last_measurement_emit = now;
        }
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                use crate::MachineApi;
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");
                sender.close();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LIVE_VALUES_EMIT_INTERVAL, STATE_EMIT_INTERVAL, should_emit_live_values,
        should_emit_state_periodically,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn periodic_state_emit_requires_autotune_or_sleep_timer() {
        let last = Instant::now();
        let now = last + STATE_EMIT_INTERVAL + Duration::from_millis(1);

        assert!(!should_emit_state_periodically(now, last, false, false));
        assert!(should_emit_state_periodically(now, last, true, false));
        assert!(should_emit_state_periodically(now, last, false, true));
    }

    #[test]
    fn periodic_state_emit_is_strictly_greater_than_interval() {
        let last = Instant::now();

        assert!(!should_emit_state_periodically(
            last + STATE_EMIT_INTERVAL,
            last,
            true,
            false
        ));
        assert!(should_emit_state_periodically(
            last + STATE_EMIT_INTERVAL + Duration::from_millis(1),
            last,
            true,
            false
        ));
    }

    #[test]
    fn live_values_emit_is_strictly_greater_than_interval() {
        let last = Instant::now();

        assert!(!should_emit_live_values(
            last + LIVE_VALUES_EMIT_INTERVAL,
            last
        ));
        assert!(should_emit_live_values(
            last + LIVE_VALUES_EMIT_INTERVAL + Duration::from_nanos(1),
            last
        ));
    }
}
