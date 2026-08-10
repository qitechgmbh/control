//! MIMO thermal control glue for [`ExtruderV2`].
//!
//! Owns the machine-level concerns that `control_core::controllers::mimo` deliberately knows
//! nothing about: which physical zone maps to which matrix index, the mode and interlock guards
//! that abort a campaign, and the handover of the four heater duties.

use super::api::{
    ExtruderV2Events, MimoEntryState, MimoGainsState, MimoIdentifyRequest, MimoModelState,
    MimoState, MimoSynthesizeRequest, MimoTraceEvent, MimoTraceSample,
};
use super::{ExtruderV2, ExtruderV2Mode, HeatingType};
use control_core::controllers::mimo::{
    MimoGains, MimoSynthesis, ZONE_COUNT,
    controller::MimoPidController,
    identify::{MimoIdentPhase, MimoIdentifyConfig},
    synth_decoupler::DecouplerImc,
};
use std::time::{Duration, Instant};

/// Zone order along the barrel, nozzle end first.
///
/// Every matrix in the MIMO path is indexed by position in this array, and it is physical order
/// rather than the order the fields happen to be declared in. Conduction is a nearest-neighbour
/// effect, so in physical order `|i - j|` is distance along the barrel and a correctly identified
/// coupling matrix comes out banded — which makes the matrix readable at a glance, keeps the
/// decoupler better conditioned, and gives a free sanity check on a campaign that took hours.
pub const ZONE_ORDER: [HeatingType; ZONE_COUNT] = [
    HeatingType::Nozzle,
    HeatingType::Front,
    HeatingType::Middle,
    HeatingType::Back,
];

/// Which scheme is driving the heating zones.
pub enum ThermalControl {
    /// Four independent SISO PID loops. The long-standing behaviour, and the default.
    Decentralized,
    /// One matrix-gain controller across all four zones.
    Mimo(Box<MimoPidController<ZONE_COUNT>>),
}

impl ThermalControl {
    pub const fn is_mimo(&self) -> bool {
        matches!(self, Self::Mimo(_))
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Decentralized => "decentralized",
            Self::Mimo(_) => "mimo",
        }
    }
}

impl ExtruderV2 {
    /// Index of a zone in [`ZONE_ORDER`].
    pub fn zone_index(zone: HeatingType) -> usize {
        ZONE_ORDER
            .iter()
            .position(|z| *z == zone)
            .expect("ZONE_ORDER covers every HeatingType")
    }

    /// Filtered temperatures of all four zones, in physical order.
    fn zone_temperatures(&self) -> [f64; ZONE_COUNT] {
        ZONE_ORDER.map(|z| self.temperature_controller(z).get_temperature_celsius())
    }

    /// Target temperatures of all four zones, in physical order.
    fn zone_setpoints(&self) -> [f64; ZONE_COUNT] {
        ZONE_ORDER.map(|z| {
            self.temperature_controller(z)
                .get_target_temperature_celsius()
        })
    }

    /// Currently commanded duties, in physical order.
    fn zone_duties(&self) -> [f64; ZONE_COUNT] {
        ZONE_ORDER.map(|z| self.temperature_controller(z).get_duty())
    }

    fn zone_max_duties(&self) -> [f64; ZONE_COUNT] {
        ZONE_ORDER.map(|z| self.temperature_controller(z).get_max_clamp())
    }

    fn any_wiring_error(&self) -> bool {
        ZONE_ORDER
            .iter()
            .any(|z| self.temperature_controller(*z).heating.wiring_error)
    }

    /// Apply a duty vector to the four zones, or hand them all back to their own PIDs.
    fn drive_zones(&mut self, duties: Option<[f64; ZONE_COUNT]>) {
        for (i, zone) in ZONE_ORDER.into_iter().enumerate() {
            self.temperature_controller_mut(zone)
                .apply_external_duty(duties.map(|d| d[i]));
        }
    }

    /// Reasons a campaign or MIMO control cannot continue that the controllers cannot see.
    ///
    /// Mirrors the guard set the single-zone tuner already uses, extended across all four zones.
    fn thermal_abort_reason(&self) -> Option<&'static str> {
        if self.any_wiring_error() {
            Some("temperature sensor wiring error")
        } else if self.mode != ExtruderV2Mode::Heat {
            Some("machine left Heat mode")
        } else if self.screw_is_turning() {
            Some("screw started turning")
        } else {
            None
        }
    }

    // ---------------------------------------------------------------- api surface

    pub fn handle_start_mimo_identification(&mut self, request: MimoIdentifyRequest) {
        let config = MimoIdentifyConfig {
            step_duty: request.step_duty,
            max_rise_celsius: request.max_rise_celsius,
            ..Default::default()
        };
        match self.start_mimo_identification(config, Instant::now()) {
            Ok(()) => self.mimo_last_error = None,
            Err(reason) => {
                tracing::warn!("mimo identification refused: {reason}");
                self.mimo_last_error = Some(reason);
            }
        }
        self.emit_state();
    }

    pub fn handle_synthesize_mimo_gains(&mut self, request: &MimoSynthesizeRequest) {
        match self.synthesize_mimo_gains(&request.method, request.lambda_factor) {
            Ok(()) => self.mimo_last_error = None,
            Err(reason) => {
                tracing::warn!("mimo synthesis refused: {reason}");
                self.mimo_last_error = Some(reason);
                self.emit_state();
            }
        }
    }

    pub fn handle_set_thermal_control_mode(&mut self, mode: &str) {
        match self.set_thermal_control_mode(mode) {
            Ok(()) => self.mimo_last_error = None,
            Err(reason) => {
                tracing::warn!("thermal control mode change refused: {reason}");
                self.mimo_last_error = Some(reason);
                self.emit_state();
            }
        }
    }

    /// Build the MIMO block of the state event.
    pub fn build_mimo_state(&self) -> MimoState {
        let zone_order: Vec<String> = ZONE_ORDER.iter().map(|z| z.as_str().to_owned()).collect();

        let model = self.mimo_model.as_ref().map(|m| MimoModelState {
            g: (0..ZONE_COUNT)
                .map(|i| {
                    (0..ZONE_COUNT)
                        .map(|j| MimoEntryState {
                            gp: m.g[i][j].gp,
                            tau: m.g[i][j].tau,
                            theta: m.g[i][j].theta,
                            rms_residual: m.g[i][j].rms_residual,
                            snr_ratio: m.g[i][j].snr_ratio,
                        })
                        .collect()
                })
                .collect(),
            zone_order: zone_order.clone(),
            setpoints: m.setpoints.to_vec(),
            rga: m.rga.iter().map(|row| row.to_vec()).collect(),
            condition_number: finite_or_zero(m.condition_number),
            niederlinski: finite_or_zero(m.niederlinski),
            max_rga_deviation: finite_or_zero(m.max_rga_deviation()),
            max_coupling_ratio: finite_or_zero(m.max_coupling_ratio()),
            identified_at_secs: m
                .identified_at
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
        });

        let gains = self.mimo_gains.as_ref().map(|g| MimoGainsState {
            kp: g.kp.iter().map(|row| row.to_vec()).collect(),
            ki: g.ki.iter().map(|row| row.to_vec()).collect(),
            kd: g.kd.iter().map(|row| row.to_vec()).collect(),
            derivative_filter_tc: g.derivative_filter_tc,
            method: self.mimo_synthesis_method.to_owned(),
        });

        let now = Instant::now();
        MimoState {
            mode: self.thermal_control.as_str().to_owned(),
            phase: self.mimo_identifier.phase().to_owned(),
            is_running: self.mimo_identifier.is_running(),
            progress_percent: self.mimo_identifier.progress_percent(now),
            elapsed_seconds: self.mimo_identifier.elapsed_seconds(now),
            column: match self.mimo_identifier.phase_enum() {
                MimoIdentPhase::Step { column } => Some(column),
                _ => None,
            },
            columns_done: self.mimo_identifier.columns_done(),
            zone_order,
            failure_reason: self
                .mimo_identifier
                .failure_reason()
                .map(str::to_owned)
                .or_else(|| self.mimo_last_error.clone()),
            model,
            gains,
            synthesis_error: self.mimo_last_error.clone(),
        }
    }

    /// Push the recorded campaign curve to clients.
    pub fn emit_mimo_trace(&mut self) {
        use control_core::socketio::namespace::NamespaceCacheingLogic;

        let event = MimoTraceEvent {
            phase: self.mimo_identifier.phase().to_owned(),
            column: match self.mimo_identifier.phase_enum() {
                MimoIdentPhase::Step { column } => Some(column),
                _ => None,
            },
            zone_order: ZONE_ORDER.iter().map(|z| z.as_str().to_owned()).collect(),
            samples: self
                .mimo_identifier
                .trace()
                .iter()
                .map(|s| MimoTraceSample {
                    t_seconds: s.t_seconds,
                    temperatures: s.pv,
                    duties: s.duty,
                })
                .collect(),
        }
        .build();
        self.namespace.emit(ExtruderV2Events::MimoTrace(event));
    }

    /// Write the current model and gains to disk. Best-effort: a failure is logged, never fatal.
    pub fn persist_mimo_record(&self) {
        if let Some(model) = &self.mimo_model {
            super::persist::write(
                &self.machine_identification_unique,
                model,
                self.mimo_gains.as_ref(),
                self.mimo_synthesis_method,
            );
        }
    }

    /// Restore a stored model and gains during construction.
    ///
    /// Deliberately does *not* enable MIMO control — see [`ThermalControl`]. Diagnostics are
    /// recomputed rather than trusted from the file, so a change to how they are derived takes
    /// effect on already-stored models.
    pub fn restore_mimo_record(&mut self) {
        let Some(record) = super::persist::read(&self.machine_identification_unique) else {
            return;
        };
        let mut model = record.model;
        model.refresh_diagnostics();
        self.mimo_model = Some(model);
        self.mimo_gains = record.gains;
        self.mimo_synthesis_method = match record.synthesis.as_str() {
            "decoupler" => "decoupler",
            "lmi" => "lmi",
            _ => "",
        };
    }

    // ---------------------------------------------------------------- identification

    /// Begin a coupling-identification campaign.
    pub fn start_mimo_identification(
        &mut self,
        mut config: MimoIdentifyConfig,
        now: Instant,
    ) -> Result<(), String> {
        if self.temperature_tuner.is_running() {
            return Err("a single-zone auto-tune is already running".into());
        }
        if self.thermal_control.is_mimo() {
            return Err(
                "switch back to decentralized control before re-identifying the coupling".into(),
            );
        }
        if let Some(reason) = self.thermal_abort_reason() {
            return Err(reason.into());
        }

        config.max_duty = self.zone_max_duties();

        // Clamp the rise limit against every zone's own cutoff, leaving margin. A campaign owns
        // all four heaters open-loop for hours, so the bound has to hold for the hottest zone, not
        // just the one being stepped.
        let headroom = ZONE_ORDER
            .iter()
            .map(|z| {
                let c = self.temperature_controller(*z);
                c.get_max_temperature_celsius() - c.get_temperature_celsius() - 20.0
            })
            .fold(f64::INFINITY, f64::min);
        if headroom <= 0.0 {
            return Err("a zone is already too close to its temperature limit".into());
        }
        config.max_rise_celsius = config.max_rise_celsius.min(headroom);
        config.max_total_rise_celsius = config.max_total_rise_celsius.min(headroom);

        let setpoints = self.zone_setpoints();
        self.mimo_identifier
            .start(config, setpoints, now)
            .map_err(str::to_owned)
    }

    pub fn stop_mimo_identification(&mut self, now: Instant) {
        self.mimo_identifier.abort("stopped by the operator", now);
        self.drive_zones(None);
        self.emit_state();
    }

    /// Advance the identification campaign and apply its commands.
    ///
    /// Runs before the zone controllers so its duties are in place for this tick, and returns
    /// whether it is driving so the caller knows not to also run MIMO control.
    pub fn tick_mimo_identifier(&mut self, now: Instant) -> bool {
        if !self.mimo_identifier.is_running() {
            return false;
        }

        if let Some(reason) = self.thermal_abort_reason() {
            self.mimo_identifier.abort(reason, now);
            self.drive_zones(None);
            self.emit_state();
            return false;
        }

        let pv = self.zone_temperatures();
        let duty = self.zone_duties();
        let command = self.mimo_identifier.update(pv, duty, now);
        self.drive_zones(command);

        if self.mimo_identifier.take_phase_changed() {
            if let Some(model) = self.mimo_identifier.take_result() {
                self.mimo_model = Some(model);
                self.persist_mimo_record();
            }
            // The 30 Hz state emission is gated on the inverter status hash, so campaign
            // transitions would otherwise never reach the UI.
            self.emit_state();
            self.emit_mimo_trace();
            self.last_mimo_trace_emit = now;
        } else if now.duration_since(self.last_mimo_trace_emit) > Duration::from_secs(5) {
            self.emit_mimo_trace();
            self.last_mimo_trace_emit = now;
        }

        command.is_some()
    }

    // ---------------------------------------------------------------- synthesis

    /// Turn the identified model into gains, without applying them.
    pub fn synthesize_mimo_gains(
        &mut self,
        method: &str,
        lambda_factor: f64,
    ) -> Result<(), String> {
        let model = self
            .mimo_model
            .as_ref()
            .ok_or("no coupling model has been identified yet")?;

        let decoupler = DecouplerImc {
            lambda_factor,
            ..Default::default()
        };
        let synth: Box<dyn MimoSynthesis> = match method {
            "decoupler" => Box::new(decoupler),
            #[cfg(feature = "mimo-lmi")]
            "lmi" => {
                use control_core::controllers::mimo::synth_lmi::{Gains, IteratedLmi, LmiConfig};
                // Warm-start from the decoupler when it will produce something: the paper's own
                // initialisation starts at near-zero gain and spends most of its iteration budget
                // climbing out, and starting from a controller that already works tends to land
                // in a better local optimum. Falls back internally if the warm start turns out to
                // violate the robustness bounds.
                let warm = decoupler.synthesize(model).ok().map(|g| Gains {
                    kp: g.kp,
                    ki: g.ki,
                    kd: g.kd,
                });
                let mean_tau =
                    (0..ZONE_COUNT).map(|i| model.g[i][i].tau).sum::<f64>() / ZONE_COUNT as f64;
                Box::new(IteratedLmi {
                    config: LmiConfig {
                        // Sample around the barrel's own dynamics rather than a fixed grid: the
                        // interesting behaviour sits within a couple of decades of 1/tau.
                        omega: control_core::controllers::mimo::synth_lmi::log_space(
                            0.01 / mean_tau,
                            100.0 / mean_tau,
                            30,
                        ),
                        tau_d: 0.1 * mean_tau,
                        ..Default::default()
                    },
                    warm_start: warm,
                })
            }
            #[cfg(not(feature = "mimo-lmi"))]
            "lmi" => {
                return Err(
                    "this build does not include the LMI synthesis backend (cargo feature \
                     `mimo-lmi`)"
                        .into(),
                );
            }
            other => return Err(format!("unknown synthesis method: {other}")),
        };

        let gains = synth.synthesize(model).map_err(|e| e.to_string())?;
        self.mimo_gains = Some(gains);
        self.mimo_synthesis_method = synth.name();
        self.persist_mimo_record();
        self.emit_state();
        Ok(())
    }

    // ---------------------------------------------------------------- control

    /// Switch between decentralized and MIMO control.
    pub fn set_thermal_control_mode(&mut self, mode: &str) -> Result<(), String> {
        match mode {
            "decentralized" => {
                self.thermal_control = ThermalControl::Decentralized;
                // Structural handover: the zones go back to their own PIDs, which were reset when
                // the MIMO controller took them.
                self.drive_zones(None);
            }
            "mimo" => {
                if self.mimo_identifier.is_running() || self.temperature_tuner.is_running() {
                    return Err("cannot switch modes while a tuning run is in progress".into());
                }
                let gains: MimoGains = self
                    .mimo_gains
                    .ok_or("no MIMO gains have been synthesized yet")?;
                if !gains.is_finite() {
                    return Err("the stored MIMO gains are not usable".into());
                }

                let mut controller = MimoPidController::from_gains(
                    gains.kp,
                    gains.ki,
                    gains.kd,
                    gains.derivative_filter_tc,
                    self.zone_max_duties(),
                );
                // Seed the integral with the load the zones are already carrying. Starting from
                // zero would drop every heater to its proportional term the instant the switch
                // happened, and a warm barrel would visibly sag before the integral rebuilt.
                controller.preload_output(&self.zone_duties());
                self.thermal_control = ThermalControl::Mimo(Box::new(controller));
            }
            other => return Err(format!("unknown thermal control mode: {other}")),
        }
        self.emit_state();
        Ok(())
    }

    /// Compute and apply this tick's MIMO duties.
    ///
    /// Reads the temperatures the zone controllers filtered on the *previous* tick, since this
    /// runs before them. At a free-running loop period against minute-scale thermal constants that
    /// lag is immaterial, and it avoids splitting `TemperatureController::update` into separate
    /// read and write halves.
    pub fn tick_mimo_control(&mut self, now: Instant) {
        if !self.thermal_control.is_mimo() {
            return;
        }

        // Fall back rather than keep driving: MIMO control assumes all four zones are healthy and
        // heating, and every one of these conditions breaks that assumption.
        if self.thermal_abort_reason().is_some() {
            self.drive_zones(None);
            return;
        }

        let setpoints = self.zone_setpoints();
        let measurements = self.zone_temperatures();
        let ThermalControl::Mimo(controller) = &mut self.thermal_control else {
            return;
        };
        let duties = controller.update(&setpoints, &measurements, now);
        self.drive_zones(Some(duties));
    }
}

/// Serde renders a non-finite float as `null`, which fails schema validation on the client. These
/// diagnostics legitimately go infinite on a singular matrix, so they are flattened here rather
/// than at every call site.
fn finite_or_zero(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}
