use anyhow::Error;

pub trait TraverseStepperAxis {
    fn set_speed(&mut self, steps_per_second: f64) -> Result<(), Error>;
    fn set_enabled(&mut self, enabled: bool);
    fn is_enabled(&self) -> bool;
    fn get_position(&self) -> i128;
    fn set_position(&mut self, position: i128);
}

pub trait TraverseEndstop {
    fn is_active(&self) -> bool;
}
