use super::stop::StopReason;
use crate::gluetex::Gluetex;
use ethercat_hal::io::analog_input::physical::AnalogInputValue;
use units::angle::degree;
use units::electric_potential::volt;
fn read_optris_voltage(input: &ethercat_hal::io::analog_input::AnalogInput) -> f64 {
    match input.get_physical() {
        AnalogInputValue::Potential(v) => v.get::<volt>(),
        _ => 0.0,
    }
}

/// Check tension arms and voltage monitors; may trigger a motors-only safety stop.
///
/// Returns whether monitor state changed (caller should emit state when true).
pub fn run_tension_and_voltage_checks(machine: &mut Gluetex) -> bool {
    let mut state_changed = false;

    // Tension arm block — each arm gets its own trigger variable so the stop
    // reason names the arm that actually fired and the accumulators don't bleed
    // into the voltage block below.
    let (winder_trigger, winder_changed) = machine.winder_tension_arm_monitor.check(
        machine.winder_tension_arm.get_angle(),
        machine.operation_mode,
    );
    state_changed |= winder_changed;

    let (tape_trigger, tape_changed) = machine.tape_feeder_tension_arm_monitor.check(
        machine.tape_feeder_tension_arm.get_angle(),
        machine.operation_mode,
    );
    state_changed |= tape_changed;

    let (inlet_trigger, inlet_changed) = machine.inlet_feeder_tension_arm_monitor.check(
        machine.inlet_feeder_tension_arm.get_angle(),
        machine.operation_mode,
    );
    state_changed |= inlet_changed;

    let ta_any_trigger = winder_trigger || tape_trigger || inlet_trigger;
    let ta_state_changed = winder_changed || tape_changed || inlet_changed;

    if ta_any_trigger && ta_state_changed {
        let reason = if winder_trigger {
            StopReason::WinderTensionArm
        } else if tape_trigger {
            StopReason::TapeFeederTensionArm
        } else {
            StopReason::InletTensionArm
        };
        tracing::warn!(
            winder_ta_deg = machine.winder_tension_arm.get_angle().get::<degree>(),
            tape_feeder_ta_deg = machine.tape_feeder_tension_arm.get_angle().get::<degree>(),
            inlet_feeder_ta_deg = machine.inlet_feeder_tension_arm.get_angle().get::<degree>(),
            winder_monitor_enabled = machine.winder_tension_arm_monitor.config.enabled,
            tape_monitor_enabled = machine.tape_feeder_tension_arm_monitor.config.enabled,
            inlet_monitor_enabled = machine.inlet_feeder_tension_arm_monitor.config.enabled,
            "tension arm monitor triggered — safety stop"
        );
        machine.apply_safety_stop(Gluetex::safety_stop_motors_only(reason));
    }

    // Voltage block — independent accumulators; reason uses the cycle-local
    // trigger variables, not the persistent .triggered fields on the monitors.
    let optris_1_voltage = read_optris_voltage(&machine.optris_1);
    let (optris_1_trigger, optris_1_changed) = machine
        .optris_1_monitor
        .check(optris_1_voltage, machine.operation_mode);
    state_changed |= optris_1_changed;

    let optris_2_voltage = read_optris_voltage(&machine.optris_2);
    let (optris_2_trigger, optris_2_changed) = machine
        .optris_2_monitor
        .check(optris_2_voltage, machine.operation_mode);
    state_changed |= optris_2_changed;

    let volt_any_trigger = optris_1_trigger || optris_2_trigger;
    let volt_state_changed = optris_1_changed || optris_2_changed;

    if volt_any_trigger && volt_state_changed {
        let reason = if optris_1_trigger {
            StopReason::Optris1Voltage
        } else {
            StopReason::Optris2Voltage
        };
        tracing::warn!(
            optris_1_v = optris_1_voltage,
            optris_2_v = optris_2_voltage,
            ?reason,
            "voltage monitor triggered — safety stop"
        );
        machine.apply_safety_stop(Gluetex::safety_stop_motors_only(reason));
    }

    state_changed
}

/// Check band monitoring; may trigger a motors-only safety stop.
///
/// Returns whether monitor state changed (caller should emit state when true).
pub fn run_bandueberwachung_check(machine: &mut Gluetex) -> bool {
    use crate::gluetex::OperationMode;

    let should_check =
        machine.bandueberwachung_enabled && machine.operation_mode == OperationMode::Production;

    if !should_check {
        machine.bandueberwachung_not_active_since = None;
        if machine.bandueberwachung_triggered {
            machine.bandueberwachung_triggered = false;
            return true;
        }
        return false;
    }

    const BAND_ACTIVE_THRESHOLD_VOLTS: f64 = 10.0;
    const BAND_DEBOUNCE_MS: u64 = 200;

    let voltage = read_optris_voltage(&machine.bandueberwachung_input);
    let active = voltage > BAND_ACTIVE_THRESHOLD_VOLTS;
    let was_triggered = machine.bandueberwachung_triggered;

    if !active {
        let since = machine
            .bandueberwachung_not_active_since
            .get_or_insert_with(std::time::Instant::now);
        if !was_triggered && since.elapsed() >= std::time::Duration::from_millis(BAND_DEBOUNCE_MS) {
            machine.bandueberwachung_triggered = true;
            tracing::warn!(voltage, "Bandüberwachung: band absent — safety stop");
            machine.apply_safety_stop(Gluetex::safety_stop_motors_only(
                StopReason::Bandueberwachung,
            ));
            return true;
        }
    } else {
        machine.bandueberwachung_not_active_since = None;
        if was_triggered {
            machine.bandueberwachung_triggered = false;
            return true;
        }
    }
    false
}

/// Check heater over-temperature after PID updates; may trigger a full safety stop.
pub fn run_heater_overtemperature_check(machine: &mut Gluetex) -> bool {
    if !machine.heaters.any_over_temperature() {
        return false;
    }

    let zone_mask = machine.heaters.over_temperature_zone_mask();

    tracing::warn!(
        zone_1_over = zone_mask & 0b000001 != 0,
        zone_2_over = zone_mask & 0b000010 != 0,
        zone_3_over = zone_mask & 0b000100 != 0,
        zone_4_over = zone_mask & 0b001000 != 0,
        zone_5_over = zone_mask & 0b010000 != 0,
        zone_6_over = zone_mask & 0b100000 != 0,
        zone_1_temp = machine.heaters.zone_temperature_celsius(0),
        zone_2_temp = machine.heaters.zone_temperature_celsius(1),
        zone_3_temp = machine.heaters.zone_temperature_celsius(2),
        zone_4_temp = machine.heaters.zone_temperature_celsius(3),
        zone_5_temp = machine.heaters.zone_temperature_celsius(4),
        zone_6_temp = machine.heaters.zone_temperature_celsius(5),
        zone_mask,
        "heater over-temperature — safety stop"
    );
    machine.apply_safety_stop(Gluetex::safety_stop_full(
        StopReason::HeaterOverTemperature { zones: zone_mask },
    ));
    machine.emit_state();
    true
}

/// Check sleep timer; may trigger a full safety stop.
pub fn run_sleep_timer_check(machine: &mut Gluetex) -> bool {
    if machine.sleep_timer.check(machine.operation_mode) {
        machine.apply_safety_stop(Gluetex::safety_stop_full(StopReason::SleepTimer));
        tracing::info!("Entered sleep mode due to inactivity");
        true
    } else {
        false
    }
}
