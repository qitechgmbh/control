#[derive(Debug)]
pub struct TraverseController {
    enabled: bool,
    is_homed: bool,
    is_going_home: bool,
    is_going_in: bool,
    is_going_out: bool,
    limit_inner: f64,
    limit_outer: f64,
    current_position: f64,
    home_position: f64,
}

impl TraverseController {
    pub fn new(limit_inner: f64, limit_outer: f64) -> Self {
        Self {
            enabled: false,
            is_homed: false,
            is_going_home: false,
            is_going_in: false,
            is_going_out: false,
            limit_inner,
            limit_outer,
            current_position: 0.0,
            home_position: 50.0, // Default home position in the middle
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_limit_inner(&mut self, limit: f64) {
        self.limit_inner = limit;
    }

    pub fn set_limit_outer(&mut self, limit: f64) {
        self.limit_outer = limit;
    }

    pub fn get_limit_inner(&self) -> f64 {
        self.limit_inner
    }

    pub fn get_limit_outer(&self) -> f64 {
        self.limit_outer
    }

    pub fn set_current_position(&mut self, position: f64) {
        todo!()
        // self.current_position = position;

        // // Update state flags based on position
        // if self.is_in() {
        //     self.is_going_in = false;
        // }
        // if self.is_out() {
        //     self.is_going_out = false;
        // }
        // if (self.current_position - self.home_position).abs() < 0.5 {
        //     self.is_going_home = false;
        //     self.is_homed = true;
        // }
    }

    pub fn get_current_position(&self) -> f64 {
        self.current_position
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_home_position(&self) -> f64 {
        self.home_position
    }

    pub fn goto_limit_inner(&mut self) {
        todo!();
        // self.is_going_in = true;
        // self.is_going_out = false;
        // self.is_going_home = false;
    }

    pub fn goto_limit_outer(&mut self) {
        todo!();
        // self.is_going_in = false;
        // self.is_going_out = true;
        // self.is_going_home = false;
    }

    pub fn goto_home(&mut self, home_position: f64) {
        todo!();
        // self.home_position = home_position;
        // self.is_going_in = false;
        // self.is_going_out = false;
        // self.is_going_home = true;
    }

    pub fn is_homed(&self) -> bool {
        self.is_homed
    }

    pub fn set_homed(&mut self, homed: bool) {
        self.is_homed = homed;
    }

    pub fn is_going_in(&self) -> bool {
        self.is_going_in
    }

    pub fn is_going_out(&self) -> bool {
        self.is_going_out
    }

    pub fn is_going_home(&self) -> bool {
        self.is_going_home
    }

    pub fn is_in(&self) -> bool {
        todo!();
        // (self.current_position - self.limit_inner).abs() < 0.5
    }

    pub fn is_out(&self) -> bool {
        todo!()
        // (self.current_position - self.limit_outer).abs() < 0.5
    }

    pub fn update_position(&mut self, time_delta: f64) {
        todo!()
        // if !self.enabled {
        //     return;
        // }

        // // Determine target position and direction
        // let target_position = if self.is_going_in {
        //     self.limit_inner
        // } else if self.is_going_out {
        //     self.limit_outer
        // } else if self.is_going_home {
        //     self.home_position
        // } else {
        //     return; // No movement needed
        // };

        // // Calculate direction and movement
        // let distance_to_target = target_position - self.current_position;
        // let direction = if distance_to_target > 0.0 { 1.0 } else { -1.0 };

        // // Apply speed (default 5.0 mm/s) and time_delta to move the position
        // let movement = direction * 5.0 * time_delta;

        // // Don't overshoot the target
        // if movement.abs() > distance_to_target.abs() {
        //     self.current_position = target_position;
        // } else {
        //     self.current_position += movement;
        // }

        // // Update state based on new position
        // self.set_current_position(self.current_position);
    }
}
