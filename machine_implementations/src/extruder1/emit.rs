#[cfg(not(feature = "mock-machine"))]
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

use crate::extruder1::{
    ExtruderV2, ExtruderV2Mode, HeatingType,
    api::{
        ExtruderSettingsState, ExtruderV2Events, HeatingPowerOverride, HeatingPowerOverrideState,
        HeatingPowerOverrideStates, HeatingState, HeatingStates, InverterStatusState,
        LiveValuesEvent, ModeState, PidAutoTuneState, PidSettings, PidSettingsStates,
        PressureAutoTuneConfig, PressureState, RegulationState, RotationState, ScrewState,
        StateEvent, TemperatureAutoTuneConfig, TemperatureAutoTuneResultState,
        TemperatureAutoTuneSample, TemperatureAutoTuneState, TemperatureAutoTuneTraceEvent,
        TemperaturePid,
    },
};

#[cfg(not(feature = "mock-machine"))]
fn power_override_state(
    controller: &crate::extruder1::temperature_controller::TemperatureController,
) -> HeatingPowerOverrideState {
    HeatingPowerOverrideState {
        enabled: controller.get_power_override_enabled(),
        watts: controller.get_power_override_watts(),
        max_watts: controller.get_max_heating_power(),
    }
}

#[cfg(not(feature = "mock-machine"))]
impl ExtruderV2 {
    pub fn get_state(&self) -> StateEvent {
        use qitech_lib::units::{
            angular_velocity::revolution_per_minute, pressure::bar,
            thermodynamic_temperature::degree_celsius,
        };

        use crate::extruder1::api::TemperaturePidStates;

        StateEvent {
            is_default_state: !self.emitted_default_state,
            rotation_state: RotationState {
                forward: self.screw_speed_controller.get_rotation_direction(),
            },
            mode_state: ModeState {
                mode: self.mode.clone(),
            },
            regulation_state: RegulationState {
                uses_rpm: self.screw_speed_controller.get_uses_rpm(),
            },
            pressure_state: PressureState {
                target_bar: self
                    .screw_speed_controller
                    .get_target_pressure()
                    .get::<bar>(),
                wiring_error: self.screw_speed_controller.wiring_error,
            },
            screw_state: ScrewState {
                target_rpm: self
                    .screw_speed_controller
                    .get_target_rpm()
                    .get::<revolution_per_minute>(),
            },
            heating_states: HeatingStates {
                nozzle: HeatingState {
                    target_temperature: self
                        .temperature_controller_nozzle
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_nozzle.heating.wiring_error,
                },
                front: HeatingState {
                    target_temperature: self
                        .temperature_controller_front
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_front.heating.wiring_error,
                },
                back: HeatingState {
                    target_temperature: self
                        .temperature_controller_back
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_back.heating.wiring_error,
                },
                middle: HeatingState {
                    target_temperature: self
                        .temperature_controller_middle
                        .heating
                        .target_temperature
                        .get::<degree_celsius>(),
                    wiring_error: self.temperature_controller_middle.heating.wiring_error,
                },
            },
            heating_power_override_states: HeatingPowerOverrideStates {
                nozzle: power_override_state(&self.temperature_controller_nozzle),
                front: power_override_state(&self.temperature_controller_front),
                back: power_override_state(&self.temperature_controller_back),
                middle: power_override_state(&self.temperature_controller_middle),
            },
            extruder_settings_state: ExtruderSettingsState {
                pressure_limit: self
                    .screw_speed_controller
                    .get_nozzle_pressure_limit()
                    .get::<bar>(),
                pressure_limit_enabled: self
                    .screw_speed_controller
                    .get_nozzle_pressure_limit_enabled(),
                nozzle_temperature_target_enabled: self
                    .temperature_controller_nozzle
                    .get_temperature_target_enabled(),
            },
            inverter_status_state: InverterStatusState {
                running: self.screw_speed_controller.inverter.status.running,
                forward_running: self.screw_speed_controller.inverter.status.forward_running,
                reverse_running: self.screw_speed_controller.inverter.status.reverse_running,
                up_to_frequency: self.screw_speed_controller.inverter.status.su,
                overload_warning: self.screw_speed_controller.inverter.status.ol,
                no_function: self.screw_speed_controller.inverter.status.no_function,
                output_frequency_detection: self.screw_speed_controller.inverter.status.fu,
                abc_fault: self.screw_speed_controller.inverter.status.abc_,
                fault_occurence: self.screw_speed_controller.inverter.status.fault_occurence,
            },
            pid_settings: PidSettingsStates {
                temperature: TemperaturePidStates {
                    front: TemperaturePid {
                        ki: self.temperature_controller_front.pid.get_ki(),
                        kp: self.temperature_controller_front.pid.get_kp(),
                        kd: self.temperature_controller_front.pid.get_kd(),
                        zone: String::from("front"),
                    },
                    middle: TemperaturePid {
                        ki: self.temperature_controller_middle.pid.get_ki(),
                        kp: self.temperature_controller_middle.pid.get_kp(),
                        kd: self.temperature_controller_middle.pid.get_kd(),
                        zone: String::from("middle"),
                    },
                    back: TemperaturePid {
                        ki: self.temperature_controller_back.pid.get_ki(),
                        kp: self.temperature_controller_back.pid.get_kp(),
                        kd: self.temperature_controller_back.pid.get_kd(),
                        zone: String::from("back"),
                    },
                    nozzle: TemperaturePid {
                        ki: self.temperature_controller_nozzle.pid.get_ki(),
                        kp: self.temperature_controller_nozzle.pid.get_kp(),
                        kd: self.temperature_controller_nozzle.pid.get_kd(),
                        zone: String::from("nozzle"),
                    },
                },
                pressure: PidSettings {
                    ki: self.screw_speed_controller.pid.get_ki(),
                    kp: self.screw_speed_controller.pid.get_kp(),
                    kd: self.screw_speed_controller.pid.get_kd(),
                },
            },
            pid_autotune_state: {
                let result = self.screw_speed_controller.get_autotune_result();
                PidAutoTuneState {
                    state: self.screw_speed_controller.get_autotune_state().to_string(),
                    progress: self.screw_speed_controller.get_autotune_progress(),
                    result,
                }
            },
            temperature_autotune_state: self.get_temperature_autotune_state(),
            mimo_state: self.build_mimo_state(),
        }
    }

    pub fn emit_state(&mut self) {
        use control_core::{
            helpers::hasher_serializer::hash_with_serde_model,
            socketio::namespace::NamespaceCacheingLogic,
        };

        let state = self.get_state();
        let hash = hash_with_serde_model(self.screw_speed_controller.get_inverter_status());
        self.last_status_hash = Some(hash);
        let event = state.build();
        self.namespace.emit(ExtruderV2Events::State(event));
        self.emitted_default_state = true;
    }

    pub fn maybe_emit_state_event(&mut self) {
        use control_core::helpers::hasher_serializer::hash_with_serde_model;

        let old_status_hash = match self.last_status_hash {
            Some(event) => event,
            None => {
                self.emit_state();
                return;
            }
        };
        let status = self.screw_speed_controller.get_inverter_status();
        let new_status_hash = hash_with_serde_model(status);
        if new_status_hash != old_status_hash {
            self.emit_state();
        }
    }

    pub fn get_live_values(&self) -> LiveValuesEvent {
        use qitech_lib::units::{pressure::bar, thermodynamic_temperature::degree_celsius};

        LiveValuesEvent {
            motor_status: self.screw_speed_controller.get_motor_status().into(),
            pressure: self.screw_speed_controller.pressure.get::<bar>(),
            nozzle_temperature: self
                .temperature_controller_nozzle
                .heating
                .temperature
                .get::<degree_celsius>(),
            front_temperature: self
                .temperature_controller_front
                .heating
                .temperature
                .get::<degree_celsius>(),
            back_temperature: self
                .temperature_controller_back
                .heating
                .temperature
                .get::<degree_celsius>(),
            middle_temperature: self
                .temperature_controller_middle
                .heating
                .temperature
                .get::<degree_celsius>(),
            nozzle_power: self
                .temperature_controller_nozzle
                .get_heating_element_wattage(),
            front_power: self
                .temperature_controller_front
                .get_heating_element_wattage(),
            back_power: self
                .temperature_controller_back
                .get_heating_element_wattage(),
            middle_power: self
                .temperature_controller_middle
                .get_heating_element_wattage(),
            combined_power: self.calculate_combined_power(),
            total_energy_kwh: self.total_energy_kwh,
        }
    }

    pub fn emit_live_values(&mut self) {
        use control_core::socketio::namespace::NamespaceCacheingLogic;

        let event = self.get_live_values().build();
        self.namespace.emit(ExtruderV2Events::LiveValues(event));
    }

    // === Steuerungsfunktionen mit emit_state ===

    pub fn set_nozzle_pressure_limit_is_enabled(&mut self, enabled: bool) {
        self.screw_speed_controller
            .set_nozzle_pressure_limit_is_enabled(enabled);
        self.emit_state();
    }

    pub fn set_nozzle_pressure_limit(&mut self, pressure: f64) {
        use qitech_lib::units::{Pressure, pressure::bar};

        self.screw_speed_controller
            .set_nozzle_pressure_limit(Pressure::new::<bar>(pressure));
        self.emit_state();
    }

    pub fn enable_heating(&mut self) {
        self.temperature_controller_back.allow_heating();
        self.temperature_controller_front.allow_heating();
        self.temperature_controller_middle.allow_heating();
        self.temperature_controller_nozzle.allow_heating();
        self.emit_state();
    }

    pub fn set_rotation_state(&mut self, forward: bool) {
        self.screw_speed_controller.set_rotation_direction(forward);
        self.emit_state();
    }

    pub fn set_mode_state(
        &mut self,
        mode: ExtruderV2Mode,
        digital_out: &mut dyn DigitalOutputDevice,
    ) {
        self.switch_mode(mode, digital_out);
        self.emit_state();
    }

    pub fn set_regulation(&mut self, uses_rpm: bool) {
        if !self.screw_speed_controller.get_uses_rpm() && uses_rpm {
            use qitech_lib::units::{AngularVelocity, angular_velocity::revolution_per_minute};

            self.screw_speed_controller.set_target_screw_rpm(
                self.screw_speed_controller.target_rpm,
                AngularVelocity::new::<revolution_per_minute>(1500.0),
                self.screw_speed_controller.motor_poles,
            );
            self.screw_speed_controller.set_uses_rpm(uses_rpm);
        }

        if self.screw_speed_controller.get_uses_rpm() && !uses_rpm {
            self.screw_speed_controller.set_uses_rpm(uses_rpm);
            self.screw_speed_controller.start_pressure_regulation();
        }
        self.emit_state();
    }

    pub fn set_target_pressure(&mut self, pressure: f64) {
        use qitech_lib::units::{Pressure, pressure::bar};

        self.screw_speed_controller
            .set_target_pressure(Pressure::new::<bar>(pressure));
        self.emit_state();
    }

    pub fn set_target_rpm(&mut self, rpm: f64) {
        use qitech_lib::units::{AngularVelocity, angular_velocity::revolution_per_minute};

        self.screw_speed_controller.set_target_screw_rpm(
            AngularVelocity::new::<revolution_per_minute>(rpm),
            AngularVelocity::new::<revolution_per_minute>(1500.0),
            self.screw_speed_controller.motor_poles,
        );
        self.emit_state();
    }

    pub fn set_target_temperature(&mut self, target_temperature: f64, heating_type: HeatingType) {
        use qitech_lib::units::{
            ThermodynamicTemperature, thermodynamic_temperature::degree_celsius,
        };

        let target_temp = ThermodynamicTemperature::new::<degree_celsius>(target_temperature);

        match heating_type {
            HeatingType::Nozzle => self
                .temperature_controller_nozzle
                .set_target_temperature(target_temp),
            HeatingType::Front => self
                .temperature_controller_front
                .set_target_temperature(target_temp),
            HeatingType::Back => self
                .temperature_controller_back
                .set_target_temperature(target_temp),
            HeatingType::Middle => self
                .temperature_controller_middle
                .set_target_temperature(target_temp),
        }
        self.emit_state();
    }

    pub fn set_nozzle_temperature_target_is_enabled(&mut self, enabled: bool) {
        self.temperature_controller_nozzle
            .set_temperature_target_enabled(enabled);
        self.emit_state();
    }

    /// Debug/test measure: drive one heating zone at a fixed wattage instead of regulating it.
    ///
    /// The zone runs open-loop while this is enabled and can overshoot its target temperature —
    /// only the per-zone maximum temperature cutoff still limits it.
    pub fn set_heating_power_override(&mut self, settings: HeatingPowerOverride) {
        let controller = match settings.zone.as_str() {
            "front" => &mut self.temperature_controller_front,
            "middle" => &mut self.temperature_controller_middle,
            "back" => &mut self.temperature_controller_back,
            "nozzle" => &mut self.temperature_controller_nozzle,
            _ => {
                tracing::warn!("Unknown zone: {}", settings.zone);
                return;
            }
        };
        controller.set_power_override(settings.enabled, settings.watts);
        if settings.enabled {
            tracing::warn!(
                "Heating zone '{}' is running at a fixed {} W (debug override) — temperature is not regulated",
                settings.zone,
                settings.watts
            );
        }
        self.emit_state();
    }

    pub fn configure_pressure_pid(&mut self, settings: PidSettings) {
        self.screw_speed_controller
            .pid
            .configure(settings.ki, settings.kp, settings.kd);
        self.emit_state();
    }

    /// Start pressure PID auto-tuning.
    ///
    /// The machine must be in `Extrude` mode and in pressure-regulation mode
    /// (`uses_rpm == false`) for the tuner to drive the actuator.
    pub fn start_pressure_pid_autotune(&mut self, config: PressureAutoTuneConfig) {
        use std::time::Instant;
        let now = Instant::now();
        self.screw_speed_controller
            .start_pressure_autotune(now, config);
        self.emit_state();
    }

    /// Abort the current pressure PID auto-tune run
    pub fn stop_pressure_pid_autotune(&mut self) {
        self.screw_speed_controller.stop_autotune();
        self.emit_state();
    }

    /// Assemble the auto-tuner's current state for the frontend.
    pub fn get_temperature_autotune_state(&self) -> TemperatureAutoTuneState {
        use std::time::Instant;

        let now = Instant::now();
        let tuner = &self.temperature_tuner;
        TemperatureAutoTuneState {
            zone: self.temperature_tuner_zone.map(|z| z.as_str().to_string()),
            phase: tuner.phase().to_string(),
            progress: tuner.progress_percent(now),
            elapsed_seconds: tuner.elapsed_seconds(now),
            baseline_duty: tuner.baseline_duty(),
            baseline_temperature: tuner.baseline_pv(),
            current_duty: tuner.commanded_duty(),
            result: tuner.result().map(|r| TemperatureAutoTuneResultState {
                process_gain: r.process_gain,
                time_constant: r.time_constant,
                dead_time: r.dead_time,
                tau_63: r.tau_63,
                dead_time_threshold: r.dead_time_threshold,
                rms_residual: r.rms_residual,
                fit_error_pct: r.fit_error_pct,
                is_good_fit: r.is_good_fit(),
                delta_pv: r.delta_pv,
                delta_u: r.delta_u,
                lambda: r.lambda,
                noise_peak_to_peak: r.noise_peak_to_peak,
                snr_ratio: r.snr_ratio,
                is_confident: r.is_confident(),
                suggested_step_duty: r.suggested_step_duty,
                pi: r.pi.into(),
                pid: r.pid.into(),
            }),
            failure_reason: tuner.failure_reason().map(str::to_string),
        }
    }

    pub fn emit_tune_trace(&mut self) {
        use control_core::socketio::namespace::NamespaceCacheingLogic;

        let event = TemperatureAutoTuneTraceEvent {
            zone: self.temperature_tuner_zone.map(|z| z.as_str().to_string()),
            phase: self.temperature_tuner.phase().to_string(),
            samples: self
                .temperature_tuner
                .trace()
                .iter()
                .map(|s| TemperatureAutoTuneSample {
                    t_seconds: s.t_seconds,
                    temperature: s.pv,
                    duty: s.duty,
                })
                .collect(),
        }
        .build();
        self.namespace.emit(ExtruderV2Events::TuneTrace(event));
    }

    /// Start an IMC step test on one heating zone.
    ///
    /// Refused unless the machine is in `Heat` mode with the screw stopped: material flow is a
    /// large, variable thermal load that an FOPDT step response cannot express.
    pub fn start_temperature_pid_autotune(&mut self, config: TemperatureAutoTuneConfig) {
        use control_core::controllers::imc_tuner::{ImcTuner, ImcTunerConfig, ImcTunerError};
        use std::time::Instant;

        let Some(zone) = HeatingType::from_zone_name(&config.zone) else {
            tracing::warn!("temperature autotune: unknown zone {}", config.zone);
            return;
        };

        if self.temperature_tuner.is_running() {
            tracing::warn!("temperature autotune: a run is already in progress");
            return;
        }
        if self.mode != ExtruderV2Mode::Heat {
            tracing::warn!("temperature autotune: requires Heat mode");
            return;
        }
        if self.screw_is_turning() {
            tracing::warn!("temperature autotune: requires the screw to be stopped");
            return;
        }

        let controller = self.temperature_controller(zone);
        if controller.heating.wiring_error {
            tracing::warn!(
                "temperature autotune: zone {} has a wiring error",
                config.zone
            );
            return;
        }
        if controller.get_power_override_enabled() {
            tracing::warn!(
                "temperature autotune: zone {} has a manual power override active",
                config.zone
            );
            return;
        }

        let baseline_duty = controller.get_duty();
        let setpoint = controller.get_target_temperature_celsius();
        let max_duty = controller.get_max_clamp();
        // Stop well short of the hard over-temperature cutoff, which is an emergency stop rather
        // than a test limit.
        let headroom_to_cutoff = controller.get_max_temperature_celsius() - 20.0 - setpoint;

        let tuner_config = ImcTunerConfig {
            step_duty: config.step_duty,
            max_duty,
            max_rise_celsius: config.max_rise_celsius.min(headroom_to_cutoff.max(1.0)),
            lambda_factor: config.lambda_factor,
            ..Default::default()
        };

        let mut tuner = ImcTuner::new(tuner_config);
        match tuner.start(Instant::now(), baseline_duty, setpoint) {
            Ok(()) => {
                self.temperature_tuner = tuner;
                self.temperature_tuner_zone = Some(zone);
                self.temperature_tuner_setpoint = setpoint;
            }
            Err(ImcTunerError::NoHeadroom { available }) => {
                tracing::warn!(
                    "temperature autotune: zone {} has only {:.3} duty of headroom, step is {:.3}",
                    config.zone,
                    available,
                    config.step_duty
                );
            }
            Err(err) => {
                tracing::warn!("temperature autotune: cannot start: {:?}", err);
            }
        }
        self.emit_state();
    }

    pub fn stop_temperature_pid_autotune(&mut self) {
        self.temperature_tuner.abort("operator stopped the run");
        self.release_temperature_tuner_zone();
        self.emit_state();
    }

    /// Push a completed run's gains into the tuned zone.
    ///
    /// The derivative filter is set from the same candidate rather than separately: a non-zero
    /// `kd` without it turns the derivative term into a noise amplifier.
    pub fn apply_temperature_autotune_result(&mut self, form: &str) {
        let Some(zone) = self.temperature_tuner_zone else {
            tracing::warn!("temperature autotune: no run to apply");
            return;
        };
        let Some(result) = self.temperature_tuner.result() else {
            tracing::warn!("temperature autotune: no result to apply");
            return;
        };
        let gains = match form {
            "pi" => result.pi,
            "pid" => result.pid,
            other => {
                tracing::warn!("temperature autotune: unknown gain form {other}");
                return;
            }
        };
        let filter_tc = gains.derivative_filter_tc();
        // The duty that was holding the zone at setpoint before the step — measured by the tune
        // itself, and exactly the standing load the integral must carry from now on.
        let baseline_duty = self.temperature_tuner.baseline_duty();
        let controller = self.temperature_controller_mut(zone);
        controller.configure_pid(gains.ki, gains.kp, gains.kd, filter_tc);
        // `configure` resets the PID, so without this the output would collapse to the (small)
        // proportional term alone and the zone would sag far below setpoint while a tiny ki
        // rebuilds the load over tens of minutes. Seed the integral with the measured load
        // instead, so regulation continues from where the tune left off.
        controller.pid.preload_integral(baseline_duty);
        self.emit_state();
    }

    fn release_temperature_tuner_zone(&mut self) {
        if let Some(zone) = self.temperature_tuner_zone {
            self.temperature_controller_mut(zone)
                .apply_external_duty(None);
        }
    }

    /// Advance the auto-tuner and apply its command to the zone under test.
    ///
    /// Called every act iteration. The machine-level guards live here because the tuner itself has
    /// no view of mode, interlocks or sensor faults.
    pub fn tick_temperature_tuner(&mut self, now: std::time::Instant) {
        use std::time::Duration;

        let Some(zone) = self.temperature_tuner_zone else {
            return;
        };
        if !self.temperature_tuner.is_running() {
            // Terminal state: make sure the zone is back under PID control. Idempotent, so it is
            // safe to run every tick.
            self.release_temperature_tuner_zone();
            return;
        }

        // Guards the tuner cannot see for itself.
        let controller = self.temperature_controller(zone);
        let pv = controller.get_temperature_celsius();
        let pid_duty = controller.get_duty();
        let wiring_error = controller.heating.wiring_error;
        let setpoint = controller.get_target_temperature_celsius();

        let abort_reason = if wiring_error {
            Some("temperature sensor wiring error")
        } else if self.mode != ExtruderV2Mode::Heat {
            Some("machine left Heat mode")
        } else if self.screw_is_turning() {
            Some("screw started turning")
        } else if (setpoint - self.temperature_tuner_setpoint).abs() > f64::EPSILON {
            Some("target temperature was changed during the run")
        } else {
            None
        };

        if let Some(reason) = abort_reason {
            self.temperature_tuner.abort(reason);
            self.release_temperature_tuner_zone();
            self.emit_state();
            return;
        }

        let command = self.temperature_tuner.update(pv, pid_duty, now);
        self.temperature_controller_mut(zone)
            .apply_external_duty(command);

        // The 30 Hz state emission is gated on the inverter status hash, so tuner transitions
        // would otherwise never reach the UI.
        if self.temperature_tuner.take_phase_changed() {
            self.emit_state();
            self.emit_tune_trace();
            self.last_tune_trace_emit = now;
        } else if self.temperature_tuner.is_running()
            && now.duration_since(self.last_tune_trace_emit) > Duration::from_secs(2)
        {
            self.emit_tune_trace();
            self.last_tune_trace_emit = now;
        }
    }

    pub fn configure_temperature_pid(&mut self, settings: TemperaturePid) {
        match settings.zone.as_str() {
            "front" => {
                self.temperature_controller_front.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            "middle" => {
                self.temperature_controller_middle.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            "back" => {
                self.temperature_controller_back.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            "nozzle" => {
                self.temperature_controller_nozzle.pid.configure(
                    settings.ki,
                    settings.kp,
                    settings.kd,
                );
            }
            _ => tracing::warn!("Unknown zone: {}", settings.zone),
        }
        self.emit_state();
    }
}
