use ethercat_hal::io::digital_output::DigitalOutput;

/// Represents the laser?
#[derive(Debug)]
pub struct Laser
{
    output: DigitalOutput,
}

impl Laser
{
    pub fn new(hardware: DigitalOutput) -> Self
    {
        Self { output: hardware }
    }

    pub fn set_toggled(&mut self, value: bool)
    {
        self.output.set(value);
    }

    pub fn is_toggled(&self) -> bool
    {
        self.output.get()
    }
}