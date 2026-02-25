use std::time::Instant;

pub struct DiameterPidController 
{
    // Gains
    kp: f64,
    ki: f64,
    kd: f64,

    // State
    integral: f64,
    previous_error: f64,
    last_update: Instant,

    // Output limits (0–75 m/min)
    min_output: f64,
    max_output: f64,

    // Integral clamp (anti-windup)
    min_integral: f64,
    max_integral: f64,
}

impl DiameterPidController
{
    pub fn new(kp: f64, ki: f64, kd: f64, min_output: f64, max_output: f64) -> Self 
    {
        Self {
            kp,
            ki,
            kd,
            integral: 0.0,
            previous_error: 0.0,
            last_update: Instant::now(),
            min_output,
            max_output,
            min_integral: -100.0,  // safe default
            max_integral: 100.0,
        }
    }

    pub fn update(&mut self, target: f64, current: f64) -> f64 
    {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f64();
        self.last_update = now;

        if dt <= 0.0 
        {
            return 0.0;
        }

        let error = target - current;

        // Integral term with clamping
        self.integral += error * dt;
        self.integral = self.integral.clamp(self.min_integral, self.max_integral);

        // Derivative term
        let derivative = (error - self.previous_error) / dt;
        self.previous_error = error;

        let output =
            self.kp * error +
            self.ki * self.integral +
            self.kd * derivative;

        // Clamp final output to 0–75 m/min
        output.clamp(self.min_output, self.max_output)
    }

    pub fn reset(&mut self) 
    {
        self.integral = 0.0;
        self.previous_error = 0.0;
        self.last_update = Instant::now();
    }
}