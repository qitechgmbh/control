#[derive(Debug)]
pub struct PidController {
    // Params
    /// Proportional gain
    kp: f32,
    /// Integral gain
    ki: f32,
    /// Derivative gain
    kd: f32,

    // State
    /// Proportional error
    ep: f32,
    /// Integral error
    ei: f32,
    /// Derivative error
    ed: f32,

    last_nanoseconds: Option<u64>,
}

impl PidController {
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            ep: 0.0,
            ei: 0.0,
            ed: 0.0,
            last_nanoseconds: None,
        }
    }

    pub fn update(&mut self, error: f32, nanoseconds: u64) -> f32 {
        let signal = match self.last_nanoseconds {
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
                self.last_nanoseconds = Some(nanoseconds);

                signal
            }
            // Subsequent updates
            Some(last_nanoseconds) => {
                // Calculate the time delta in seconds, handling potential overflow
                let dt = if nanoseconds > last_nanoseconds {
                    (nanoseconds - last_nanoseconds) as f32 / 1_000_000_000.0
                } else {
                    // default to 1 ns
                    // This is a fallback to prevent division by zero or negative time
                    1e-9
                };

                // Calculate errors
                let ep = error;
                let ei = self.ei + ep * dt;
                let ed = (ep - self.ep) / dt;

                // Calculate signal
                let signal = self.kp * ep + self.ki * ei + self.kd * ed;

                // Set values
                self.ep = ep;
                self.ei = ei;
                self.ed = ed;
                self.last_nanoseconds = Some(nanoseconds);

                signal
            }
        };

        signal
    }

    pub fn reset(&mut self) {
        self.ep = 0.0;
        self.ei = 0.0;
        self.ed = 0.0;
        self.last_nanoseconds = None;
    }
}
