use std::time::Duration;

use serde::Serialize;

use control_core_derive::BuildEvent;
use control_core::socketio::{
    event::{BuildEvent, GenericEvent}, 
    namespace::{CacheFn, CacheableEvents, cache_duration}
};

use units::{
    angle::degree, 
    angular_velocity::revolution_per_minute, 
    length::{meter, millimeter}, 
    velocity::meter_per_minute
};

use crate::winder::Winder;

#[derive(Serialize, Debug, Clone, Default, BuildEvent)]
pub struct LiveValues
{
    /// traverse position in mm
    pub traverse_position: Option<f64>,
    /// puller speed in m/min
    pub puller_speed: f64,
    /// spool rpm
    pub spool_rpm: f64,
    /// tension arm angle in degrees
    pub tension_arm_angle: f64,
    // task progress in meters (pulled distance of filament)
    pub spool_length_task_progress: f64,
}

impl CacheableEvents<Self> for LiveValues 
{
    fn event_value(&self) -> GenericEvent 
    {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn 
    {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

impl Winder
{
    pub fn create_live_values(&self) -> LiveValues
    {
        let spool_rpm = 
            self.spool.speed().get::<revolution_per_minute>();

        let puller_speed = 
            self.puller.output_speed().get::<meter_per_minute>();

        let traverse_position = 
            self.traverse.current_position().map(|x| x.get::<millimeter>());

        let tension_arm_angle = self.tension_arm.angle().get::<degree>();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let tension_arm_angle = match tension_arm_angle >= 270.0
        {
            true  => tension_arm_angle - 360.0,
            false => tension_arm_angle,
        };

        let spool_length_task_progress = 
            self.spool_length_task.progress().get::<meter>();

        LiveValues {
            spool_rpm,
            traverse_position,
            puller_speed,
            tension_arm_angle,
            spool_length_task_progress,
        }
    }
}