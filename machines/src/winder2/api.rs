
use std::time::Duration;

use control_core::socketio::namespace::CacheFn;
use control_core::socketio::namespace::CacheableEvents;
use control_core::socketio::namespace::cache_duration;
use control_core_derive::BuildEvent;
use control_core::socketio::event::{BuildEvent, GenericEvent};
use serde::{Deserialize, Serialize};

// use smol::channel::Sender

use crate::winder2::devices::PullerGearRatio;
use crate::winder2::devices::SpoolSpeedControlMode;

use crate::{MachineApi, MachineMessage};
use crate::{MachineCrossConnectionState, machine_identification::MachineIdentificationUnique};

use super::Mode;

// Keep for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpoolSpeedControllerType 
{
    Adaptive,
    MinMax,
}

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
    pub spool_progress: f64,
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

// State
#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct State 
{
    pub is_default_state: bool,
    /// traverse state
    pub traverse_state: TraverseState,
    /// puller state
    pub puller_state: PullerState,
    /// spool automatic action state and progress
    pub spool_automatic_action_state: SpoolAutomaticActionState,
    /// mode state
    pub mode_state: ModeState,
    /// tension arm state
    pub tension_arm_state: TensionArmState,
    /// spool speed controller state
    pub spool_speed_controller_state: SpoolSpeedControllerState,
    /// Is a Machine Connected?
    pub connected_machine_state: MachineCrossConnectionState,
}

impl CacheableEvents<Self> for State 
{
    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseState 
{
    /// min position in mm
    pub limit_inner: f64,
    /// max position in mm
    pub limit_outer: f64,
    /// position in mm
    pub position_in: f64,
    /// position out in mm
    pub position_out: f64,
    /// is going to position in
    pub is_going_in: bool,
    /// is going to position out
    pub is_going_out: bool,
    /// if is homed
    pub is_homed: bool,
    /// if is homing
    pub is_going_home: bool,
    /// if is traversing
    pub is_traversing: bool,
    /// laserpointer is on
    pub laserpointer: bool,
    /// step size in mm
    pub step_size: f64,
    /// padding in mm
    pub padding: f64,
    /// can go in (to inner limit)
    pub can_go_in: bool,
    /// can go out (to outer limit)
    pub can_go_out: bool,
    /// can home
    pub can_go_home: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct PullerState 
{
    /// regulation type
    pub regulation: PullerRegulationMode,
    /// target speed in m/min
    pub target_speed: f64,
    /// target diameter in mm
    pub target_diameter: f64,
    /// forward rotation direction
    pub forward: bool,
    /// gear ratio for winding speed
    pub gear_ratio: GearRatio,
}

