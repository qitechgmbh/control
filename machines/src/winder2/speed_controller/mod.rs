pub enum SpeedController
{
    Fixed(FixedSpeedController),
    Dynamic(DynamicSpeedController),
}

pub struct FixedSpeedController
{
    prev_speed:   Velocity,
    target_speed: Velocity,
}

pub enum DynamicSpeedController
{
    Laser(LaserSpeedController)
}

pub struct LaserSpeedController
{
    base_speed:       Length,
    current_diameter: Length,
    target_diameter:  Length,
}

impl LaserSpeedController
{
    pub fn update(&mut self) -> Velocity
    {
        todo!()
    }

    pub fn set_current_diamater(&mut self, value: Length)
    {
        self.current_diameter = value;
    }

    pub fn set_target_diameter(&mut self, value: Length)
    {
        self.target_diameter = value;
    }
}