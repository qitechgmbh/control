pub struct MotorConverter {}
impl MotorConverter {
    pub fn rpm_to_hz(rpm: f32) -> f32 {
        rpm / 60.0
    }
    pub fn hz_to_rpm(hz: f32) -> f32 {
        hz * 60.0
    }
}
