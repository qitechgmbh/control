use ethercat_hal::io::digital_output::DigitalOutput;

/// Represents the laser?
pub struct Laser
{
    hardware_interface: DigitalOutput,
}

impl Laser
{
    pub fn new(hardware_interface: DigitalOutput) -> Self
    {
        Self { hardware_interface }
    }

    pub fn toggled(&self) -> bool
    {
        self.hardware_interface.get()
    }
}