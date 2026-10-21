use std::time::Duration;
use std::time::Instant;

use qitech_lib::units::VolumeRate;
use qitech_lib::units::volume_rate::liter_per_minute;

use super::config::ControllerConfig;
use super::io::FlowSensor;
use super::io::Relay;
use super::timing::Timer;

/// What happened during a pump update that the owning controller still has to act on.
#[derive(Debug, Clone, Copy, Default)]
pub struct PumpOutcome {
    /// The pump command changed on its own, so PID and PWM timing are stale.
    pub control_reset: bool,
    /// Flow stayed below the thermal minimum for too long and the pump was stopped.
    pub low_flow_trip: bool,
}

/// Owns the pump relay, the flow probe, and every timer that gates flow.
pub struct PumpController {
    relay: Relay,
    flow_sensor: FlowSensor,
    flow: VolumeRate,
    /// Machine-level permission; revoking it stops the pump immediately.
    allowed: bool,
    /// The operator/mode request for flow.
    requested: bool,
    running_since: Timer,
    low_flow_since: Timer,
    cooldown: Timer,
    flow_valid_since: Timer,
}

impl PumpController {
    pub fn new(relay: Relay, flow_sensor: FlowSensor) -> Self {
        Self {
            relay,
            flow_sensor,
            flow: VolumeRate::new::<liter_per_minute>(0.0),
            allowed: false,
            requested: false,
            running_since: Timer::default(),
            low_flow_since: Timer::default(),
            cooldown: Timer::default(),
            flow_valid_since: Timer::default(),
        }
    }

    pub fn allow(&mut self) {
        self.allowed = true;
    }

    pub fn flow(&self) -> VolumeRate {
        self.flow
    }

    pub fn is_requested(&self) -> bool {
        self.requested
    }

    /// Reports whether the command actually changed, since a change invalidates the
    /// accumulated control history.
    pub fn set_requested(&mut self, requested: bool) -> bool {
        let changed = self.requested != requested;
        self.requested = requested;
        changed
    }

    /// The relay is closed *and* flow is still wanted — the state every thermal
    /// interlock is gated on. A pump coasting through its shutdown cooldown is
    /// deliberately not "delivering": the heater must already be off by then.
    pub fn is_delivering(&self) -> bool {
        self.relay.is_on() && self.requested
    }

    pub fn has_thermal_flow(&self, config: &ControllerConfig) -> bool {
        self.flow >= config.min_flow_for_thermal
    }

    /// Flow has been present long enough for the heater to be allowed on.
    pub fn flow_is_settled(&self, now: Instant, config: &ControllerConfig) -> bool {
        self.flow_valid_since
            .has_elapsed(now, config.thermal_flow_settle_duration)
    }

    /// Counts down while an unsettled loop is holding the heater off.
    pub fn settle_remaining(&self, now: Instant, config: &ControllerConfig) -> Duration {
        self.flow_valid_since
            .remaining(now, config.thermal_flow_settle_duration)
            .unwrap_or(config.thermal_flow_settle_duration)
    }

    /// Counts down while the pump is running purely to flush residual heat.
    pub fn cooldown_remaining(&self, now: Instant, config: &ControllerConfig) -> Duration {
        self.cooldown
            .remaining(now, config.thermal_flow_settle_duration)
            .unwrap_or(Duration::ZERO)
    }

    pub fn is_cooling_down(&self, now: Instant, config: &ControllerConfig) -> bool {
        self.cooldown.is_running()
            && !self
                .cooldown
                .has_elapsed(now, config.thermal_flow_settle_duration)
    }

    pub fn update(
        &mut self,
        now: Instant,
        config: &ControllerConfig,
        heater_recently_active: bool,
    ) -> PumpOutcome {
        let mut outcome = PumpOutcome::default();

        self.flow = self.flow_sensor.read();
        let has_thermal_flow = self.has_thermal_flow(config);

        // Flow failed while the pump was supposed to be delivering. Give it time to
        // prime first, then a further grace period, before giving up on it.
        let priming_done = self
            .running_since
            .has_elapsed(now, config.pump_startup_grace_period);
        if self.is_delivering() && !has_thermal_flow && priming_done {
            let started_at = self.low_flow_since.start_if_idle(now);
            if now.duration_since(started_at) >= config.low_flow_grace_period {
                self.low_flow_since.clear();
                outcome.control_reset = self.set_requested(false);
                outcome.low_flow_trip = true;
            }
        } else {
            self.low_flow_since.clear();
        }

        // Standby must not strand heat in a stagnant loop: once the heater has been on,
        // the pump keeps circulating for the settle duration before it may stop.
        if !self.requested && self.relay.is_on() && heater_recently_active {
            self.cooldown.start_if_idle(now);
        } else if !self.relay.is_on() || self.requested {
            self.cooldown.clear();
        }

        let keep_running = self.requested || self.is_cooling_down(now, config);
        if !self.relay.is_on() && self.allowed && keep_running {
            self.relay.set(true);
            self.running_since.restart(now);
        } else if self.relay.is_on() && (!self.allowed || !keep_running) {
            self.relay.set(false);
            self.running_since.clear();
            self.cooldown.clear();
        }

        // Tracked after actuation, so a pump that just stopped restarts the settle wait.
        if self.is_delivering() && has_thermal_flow {
            self.flow_valid_since.start_if_idle(now);
        } else {
            self.flow_valid_since.clear();
        }

        outcome
    }
}
