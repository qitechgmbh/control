use std::time::Instant;

#[derive(Debug)]
pub struct PidController {
    // Params
    /// Proportional gain
    kp: f64,
    /// Integral gain
    ki: f64,
    /// Derivative gain
    kd: f64,
    // State
    /// Proportional error
    ep: f64,
    /// Integral error
    ei: f64,
    /// Derivative error
    ed: f64,

    last: Option<Instant>,
}

impl PidController {
    pub const fn new(kp: f64, ki: f64, kd: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            last: None,
        }
    }

    pub const fn configure(&mut self, ki: f64, kp: f64, kd: f64) {
        self.reset();
        self.kp = kp;
        self.ki = ki;
        self.kd = kd;
    }

    pub const fn get_kp(&self) -> f64 {
        self.kp
    }

    pub const fn get_ki(&self) -> f64 {
        self.ki
    }

    pub const fn get_kd(&self) -> f64 {
        self.kd
    }

    pub fn update(&mut self, error: f64, t: Instant) -> f64 {
        match self.last {
            // First update
            None => {
                // Calculate error
                let ep = error;

                // Calculate signal
                let signal = self.kp * ep;

                // Set values
                self.ep = ep;
                self.ei = 0.0;

                self.ed = 0.0;
                self.last = Some(t);

                signal
            }
            // Subsequent updates
            Some(last) => {
                // Calculate the time delta in seconds
                let dt = t.duration_since(last).as_secs_f64();

                // Calculate errors
                let ep = error;
                let ei = ep.mul_add(dt, self.ei);
                let ed = (ep - self.ep) / dt;

                // Calculate signal
                let signal = self.kd.mul_add(ed, self.kp.mul_add(ep, self.ki * ei));

                // Set values
                self.ep = ep;
                self.ei = ei;
                self.ed = ed;
                self.last = Some(t);

                signal
            }
        }
    }

    /// Like [`Self::update`], but with conditional-integration anti-windup.
    ///
    /// While the un-clamped output is saturated and the error still points the
    /// same way, this tick's integral increment is undone, so `ei` freezes
    /// instead of winding up. That makes a non-zero `ki` safe for plants that
    /// start far from the setpoint and saturate for a long time (e.g. the
    /// extruder heaters).
    ///
    /// `out_min`/`out_max` are the caller's own clamp bounds. The returned
    /// value is `update`'s raw signal clamped to those bounds.
    pub fn update_with_antiwindup(
        &mut self,
        error: f64,
        t: Instant,
        out_min: f64,
        out_max: f64,
    ) -> f64 {
        let dt = self
            .last
            .map(|last| t.duration_since(last).as_secs_f64())
            .unwrap_or(0.0);

        let raw = self.update(error, t);
        let clamped = raw.clamp(out_min, out_max);

        if self.ki != 0.0 {
            let winding_up = clamped >= out_max && error > 0.0;
            let winding_down = clamped <= out_min && error < 0.0;
            if winding_up || winding_down {
                // Undo the integration `update` just performed for this tick.
                self.ei -= error * dt;
            }
        }

        clamped
    }

    pub const fn reset(&mut self) {
        self.ep = 0.0;
        self.ei = 0.0;
        self.ed = 0.0;
        self.last = None;
    }
}
