use control_core::socketio::{event::GenericEvent, namespace::{CacheFn, CacheableEvents}};
use control_core_derive::BuildEvent;
use serde::Serialize;
use units::{length::{meter, millimeter}, velocity::meter_per_minute};

use crate::{
    types::Direction, 
    winder2::{
        Winder2, 
        devices::{
            PullerGearRatio, 
            PullerSpeedRegulation, 
            SpoolSpeedControlMode
        }, types::SpoolLengthTaskCompletedAction
    }
};

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
    pub spool_automatic_action_state: SpoolLengthTaskState,
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
pub struct ModeState 
{
    /// mode
    pub mode: Mode,
    /// can wind
    pub can_wind: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TensionArmState 
{
    /// is zeroed/calibrated
    pub zeroed: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolLengthTaskState 
{
    pub spool_required_meters: f64,
    pub spool_automatic_action_mode: SpoolLengthTaskCompletedAction,
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolState
{
    /// regulation mode
    pub regulation_mode: SpoolSpeedControlMode,
    /// min speed in rpm for minmax mode
    pub minmax_min_speed: f64,
    /// max speed in rpm for minmax mode
    pub minmax_max_speed: f64,
    /// tension target for adaptive mode (0.0-1.0)
    pub adaptive_tension_target: f64,
    /// radius learning rate for adaptive mode
    pub adaptive_radius_learning_rate: f64,
    /// max speed multiplier for adaptive mode
    pub adaptive_max_speed_multiplier: f64,
    /// acceleration factor for adaptive mode
    pub adaptive_acceleration_factor: f64,
    /// deacceleration urgency multiplier for adaptive mode
    pub adaptive_deacceleration_urgency_multiplier: f64,
    /// forward rotation direction
    pub forward: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerState 
{
    /// regulation type
    pub regulation: PullerSpeedRegulation,
    /// target speed in m/min
    pub target_speed: f64,
    /// target diameter in mm
    pub target_diameter: f64,
    /// forward rotation direction
    pub forward: bool,
    /// gear ratio for winding speed
    pub gear_ratio: PullerGearRatio,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseState {
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

// state event
impl Winder2
{
    // COMPLETE
    fn create_tension_arm_state(&self) -> TensionArmState
    {
        TensionArmState {
            zeroed: self.tension_arm.is_calibrated()
        }
    }

    //TODO: finish
    fn create_spool_state(&self) -> SpoolState
    {
        SpoolState {
            regulation_mode:  self.spool.speed_control_mode(),
            forward: todo!(),
            // min max speed controller
            minmax_min_speed: todo!(),
            minmax_max_speed: todo!(),
            // adaptive speed controller
            adaptive_tension_target: todo!(),
            adaptive_radius_learning_rate: todo!(),
            adaptive_max_speed_multiplier: todo!(),
            adaptive_acceleration_factor: todo!(),
            adaptive_deacceleration_urgency_multiplier: todo!(),
        }
    }

    // COMPLETE
    fn create_puller_state(&self) -> PullerState
    {
        let puller = &self.puller;

        PullerState {
            regulation:      puller.speed_regulation_mode(),
            target_speed:    puller.target_speed().get::<meter_per_minute>(),
            target_diameter: puller.target_diameter().get::<millimeter>(),
            forward:         puller.direction() == Direction::Forward,
            gear_ratio:      puller.gear_ratio(),
        }
    }

    // COMPLETE
    fn create_traverse_state(&self) -> TraverseState
    {
        // NOTE(JSE): why is limit_inner and position_in identical?

        let traverse = &self.traverse;

        TraverseState { 
            limit_inner:   traverse.limit_inner().get::<millimeter>(), 
            limit_outer:   traverse.limit_outer().get::<millimeter>(),  
            position_in:   traverse.limit_inner().get::<millimeter>(),  
            position_out:  traverse.limit_outer().get::<millimeter>(),  
            is_going_in:   traverse.is_going_in(), 
            is_going_out:  traverse.is_going_out(), 
            is_homed:      traverse.is_homed(), 
            is_going_home: traverse.is_going_home(), 
            is_traversing: traverse.is_traversing(), 
            laserpointer:  traverse.laser_pointer_enabled(),
            step_size:     traverse.step_size().get::<millimeter>(), 
            padding:       traverse.padding().get::<millimeter>(), 
            can_go_in:     traverse.can_goto_limit_inner(), 
            can_go_out:    traverse.can_goto_limit_outer(), 
            can_go_home:   traverse.can_go_home() 
        }
    }

    // COMPLETE
    fn create_spool_length_task_state(&self) -> SpoolLengthTaskState
    {
        let spool_required_meters = self.spool_length_task.target_length().get::<meter>();

        SpoolLengthTaskState {
            spool_required_meters,
            spool_automatic_action_mode: self.on_spool_length_task_complete,
        }
    }
}
