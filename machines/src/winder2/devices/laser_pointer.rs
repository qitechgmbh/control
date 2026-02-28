use ethercat_hal::io::digital_output::DigitalOutput;

/// Represents the traverses laser pointer
#[derive(Debug)]
pub struct LaserPointer
{
    output: DigitalOutput,
}

impl LaserPointer
{
    pub fn new(output: DigitalOutput) -> Self
    {
        Self { output }
    }

    pub fn set_enabled(&mut self, value: bool)
    {
        self.output.set(value);
    }

    pub fn is_enabled(&self) -> bool
    {
        self.output.get()
    }
}