use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use super::digital_input::DigitalInput;

/// ScioSense UFM-02 size variant, determines volume per pulse.
///
/// From datasheet Table 7.
#[derive(Debug, Clone, Copy)]
pub enum Ufm02Size {
    /// 3/8" — 2 ml/pulse, 500 pulses/l
    Inch3_8,
    /// 1/2" — 5 ml/pulse, 200 pulses/l
    Inch1_2,
    /// 3/4" — 8 ml/pulse, 125 pulses/l
    Inch3_4,
    /// 1" — 20 ml/pulse, 50 pulses/l
    Inch1,
    /// 1.5" — 50 ml/pulse, 20 pulses/l
    Inch1_5,
}

impl Ufm02Size {
    pub const fn ml_per_pulse(&self) -> f64 {
        match self {
            Self::Inch3_8 => 2.0,
            Self::Inch1_2 => 5.0,
            Self::Inch3_4 => 8.0,
            Self::Inch1 => 20.0,
            Self::Inch1_5 => 50.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UfmFlowData {
    /// Flow rate in liters per hour, averaged over the measurement window.
    pub flow_lph: f64,
    /// Accumulated total volume in cubic meters since this instance was created.
    pub total_volume_m3: f64,
    /// True when the sensor signals an error (IO1 active-LOW: no water / low signal amplitude).
    pub error: bool,
}

/// UFM-02 ultrasonic flow sensor read via two EL1124 digital inputs.
///
/// Wiring (both outputs are open-drain, pulled up to VCC via 4k7 resistor):
/// - `pulse_input`  ← UFM-02 IO0 (DI1): idle HIGH, one falling edge per volume unit
/// - `error_input`  ← UFM-02 IO1 (DI2): idle HIGH, LOW = error (no water / low amplitude)
///
/// Call [`tick`] once every EtherCAT cycle. The maximum UFM-02 pulse rate is 111 Hz (9 ms
/// period), well within a 1 ms EtherCAT cycle rate, so no pulses are missed.
pub struct UfmFlowInput {
    pulse_input: DigitalInput,
    error_input: DigitalInput,
    ml_per_pulse: f64,
    prev_pulse: bool,
    pulse_timestamps: VecDeque<Instant>,
    /// Sliding window used for flow-rate averaging.
    window: Duration,
    total_pulses: u64,
}

impl UfmFlowInput {
    /// `window` controls the flow-rate averaging period.
    /// 5 seconds is a good default: responsive enough for control, stable enough to avoid noise.
    pub const DEFAULT_WINDOW: Duration = Duration::from_secs(5);

    pub fn new(pulse_input: DigitalInput, error_input: DigitalInput, size: Ufm02Size) -> Self {
        Self::with_window(pulse_input, error_input, size, Self::DEFAULT_WINDOW)
    }

    pub fn with_window(
        pulse_input: DigitalInput,
        error_input: DigitalInput,
        size: Ufm02Size,
        window: Duration,
    ) -> Self {
        Self {
            pulse_input,
            error_input,
            ml_per_pulse: size.ml_per_pulse(),
            // Open-drain idles HIGH, so initial state is true.
            prev_pulse: true,
            pulse_timestamps: VecDeque::new(),
            window,
            total_pulses: 0,
        }
    }

    /// Update internal state from the current EtherCAT cycle and return the latest readings.
    ///
    /// `now` should be the cycle timestamp passed into `MachineAct::act`.
    pub fn tick(&mut self, now: Instant) -> Result<UfmFlowData, anyhow::Error> {
        let current_pulse = self.pulse_input.get_value()?;
        let error_pin = self.error_input.get_value()?;

        // Detect falling edge: open-drain output goes LOW when a pulse occurs.
        if self.prev_pulse && !current_pulse {
            self.total_pulses += 1;
            self.pulse_timestamps.push_back(now);
        }
        self.prev_pulse = current_pulse;

        // Evict timestamps that have fallen outside the measurement window.
        let cutoff = now.checked_sub(self.window).unwrap_or(now);
        while self
            .pulse_timestamps
            .front()
            .map(|t| *t <= cutoff)
            .unwrap_or(false)
        {
            self.pulse_timestamps.pop_front();
        }

        // flow rate = (pulses_in_window × ml/pulse) / window_s  →  ml/s  →  ×3.6  →  l/h
        let pulses_in_window = self.pulse_timestamps.len() as f64;
        let flow_lph = pulses_in_window * self.ml_per_pulse / (self.window.as_secs_f64() * 1000.0)
            * 3600.0;

        let total_volume_m3 = self.total_pulses as f64 * self.ml_per_pulse / 1_000_000.0;

        // IO1 is active-LOW: pin LOW (false) means error.
        let error = !error_pin;

        Ok(UfmFlowData {
            flow_lph,
            total_volume_m3,
            error,
        })
    }

    /// Total number of pulses counted since creation (useful for persistence / debugging).
    pub fn total_pulses(&self) -> u64 {
        self.total_pulses
    }
}
