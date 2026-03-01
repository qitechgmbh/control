use std::time::Duration;

use control_core::socketio::{event::{BuildEvent, GenericEvent}, namespace::{CacheFn, CacheableEvents, cache_duration}};
use control_core_derive::BuildEvent;
use serde::Serialize;
use units::{angle::degree, angular_velocity::revolution_per_minute, length::{meter, millimeter}, velocity::meter_per_minute};

use crate::winder2::Winder2;

mod state;
pub use state::State;

mod mutation;

// Live values
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
    // spool progress in meters (pulled distance of filament)
    pub spool_length_task_progress: f64,
}

impl CacheableEvents<Self> for LiveValues 
{
    fn event_value(&self) -> GenericEvent 
    {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

impl Winder2
{
    pub fn create_live_values(&self) -> LiveValues
    {
        let tension_arm_angle = self.tension_arm.get_angle().get::<degree>();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let tension_arm_angle = match tension_arm_angle >= 270.0
        {
            true  => tension_arm_angle - 360.0,
            false => tension_arm_angle,
        };

        LiveValues {
            traverse_position: self.traverse.current_position().map(|x| x.get::<millimeter>()),
            puller_speed: self.puller.motor_speed().get::<meter_per_minute>(),
            spool_rpm: self.spool.speed().get::<revolution_per_minute>(),
            spool_length_task_progress: self.spool_length_task.current_length().get::<meter>(),
            tension_arm_angle,
        }
    }
}