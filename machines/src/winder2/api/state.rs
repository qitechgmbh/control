use control_core::socketio::{event::{BuildEvent, GenericEvent}, namespace::{CacheFn, CacheableEvents, cache_first_and_last_event}};
use control_core_derive::BuildEvent;
use serde::Serialize;
use units::{angular_velocity::revolution_per_minute, length::{meter, millimeter}, velocity::meter_per_minute};

use crate::{
    MachineCrossConnectionState, types::Direction, winder2::{
        Winder2, 
        devices::{
            PullerGearRatio, PullerSpeedControlMode, SpoolSpeedControlMode
        }, types::{Mode, SpoolLengthTaskCompletedAction}
    }
};


#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct State 
{
    pub is_default_state: bool,
    pub mode: Mode,
    pub can_wind: bool,
    pub spool_state: SpoolState,
    pub puller_state: PullerState,
    pub traverse_state: TraverseState,
    pub tension_arm_state: TensionArmState,
    pub spool_length_task_state: SpoolLengthTaskState,
    pub puller_adaptive_reference_machine: MachineCrossConnectionState,
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

#[derive(Serialize, Debug, Clone)]
pub struct SpoolState
{
    pub direction: Direction,
    pub speed_control_mode: SpoolSpeedControlMode,

    /// min max mode
    pub minmax_min_speed: f64, // in rpm
    pub minmax_max_speed: f64, // in rpm

    // adaptive mode
    pub adaptive_tension_target: f64,
    pub adaptive_radius_learning_rate: f64,
    pub adaptive_max_speed_multiplier: f64,
    pub adaptive_acceleration_factor: f64,
    pub adaptive_deacceleration_urgency_multiplier: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct PullerState 
{
    pub direction: Direction,
    pub gear_ratio: PullerGearRatio,
    pub speed_control_mode: PullerSpeedControlMode,

    // fixed speed strategy
    pub fixed_target_speed: f64, // in m/min

    // adaptive speed strategy
    pub adaptive_base_speed: f64, // in m/min
    pub adaptive_deviation_max: f64, // in m/min
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseState {
    // config
    pub limit_inner:  f64, /// in mm
    pub limit_outer:  f64, /// in mm
    pub position_in:  f64, /// in mm
    pub position_out: f64, /// in mm
    pub step_size:    f64, /// in mm
    pub padding:      f64, /// in mm

    // states
    pub is_going_in: bool,
    pub is_going_out: bool,
    pub is_homed: bool,
    pub is_going_home: bool,
    pub is_traversing: bool,

    // state transitions
    pub can_go_in:  bool,
    pub can_go_out:  bool,
    pub can_go_home: bool,

    // lazeeeeeeeer
    pub laserpointer_enabled: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TensionArmState 
{
    pub is_calibrated: bool,
}

#[derive(Serialize, Debug, Clone)]
pub struct SpoolLengthTaskState 
{
    pub target_length: f64,
    pub on_completed_action: SpoolLengthTaskCompletedAction,
}

impl Winder2
{
    pub fn create_state(&self) -> State
    {
        let is_default_state = !self.emitted_default_state;

        let connected_machine_state = match &self.puller_speed_reference_machine
        {
            Some(connection) => MachineCrossConnectionState { 
                machine_identification_unique: Some(connection.ident.clone()),
                is_available: true
            },
            None => MachineCrossConnectionState { 
                machine_identification_unique: None,
                is_available: false
            },
        };

        State {
            is_default_state,
            mode: self.mode, 
            can_wind: self.can_wind(),
            traverse_state: self.create_traverse_state(),
            puller_state: self.create_puller_state(),
            spool_length_task_state: self.create_spool_length_task_state(),
            tension_arm_state: self.create_tension_arm_state(),
            spool_state: self.create_spool_state(),
            puller_adaptive_reference_machine: connected_machine_state,
        }
    }

    fn create_tension_arm_state(&self) -> TensionArmState
    {
        TensionArmState {
            is_calibrated: self.tension_arm.is_calibrated()
        }
    }

    fn create_spool_state(&self) -> SpoolState
    {
        use revolution_per_minute as rpm;

        let speed_controllers = &self.spool.speed_controllers;
        let minmax   = &speed_controllers.minmax;
        let adaptive = &speed_controllers.adaptive;

        let adaptive_deacceleration_urgency_multiplier = 
            adaptive.deacceleration_urgency_multiplier();

        SpoolState {
            direction: self.spool.direction(),
            speed_control_mode: self.spool.speed_control_mode(),
            // min max speed controller
            minmax_min_speed: minmax.min_speed().get::<rpm>(),
            minmax_max_speed: minmax.max_speed().get::<rpm>(),
            // adaptive speed controller
            adaptive_tension_target: adaptive.tension_target(),
            adaptive_radius_learning_rate: adaptive.tension_target(),
            adaptive_max_speed_multiplier: adaptive.max_speed_multiplier(),
            adaptive_acceleration_factor:  adaptive.acceleration_factor(),
            adaptive_deacceleration_urgency_multiplier,
        }
    }

    fn create_puller_state(&self) -> PullerState
    {
        let puller = &self.puller;

        let strategies = puller.speed_controller_strategies();

        let fixed_target_speed = 
            strategies.fixed.target_speed().get::<meter_per_minute>();

        let adaptive_base_speed = 
            strategies.adaptive.base_speed().get::<meter_per_minute>();

        let adaptive_deviation_max = 
            strategies.adaptive.deviation_max().get::<meter_per_minute>();

        PullerState {
            direction:  puller.direction(),
            gear_ratio: puller.gear_ratio(),
            speed_control_mode: puller.speed_control_mode(),
            fixed_target_speed,
            adaptive_base_speed,
            adaptive_deviation_max,
        }
    }

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
            laserpointer_enabled:  traverse.laser_pointer_enabled(),
            step_size:     traverse.step_size().get::<millimeter>(), 
            padding:       traverse.padding().get::<millimeter>(), 
            can_go_in:     traverse.can_goto_limit_inner(), 
            can_go_out:    traverse.can_goto_limit_outer(), 
            can_go_home:   traverse.can_go_home() 
        }
    }

    fn create_spool_length_task_state(&self) -> SpoolLengthTaskState
    {
        SpoolLengthTaskState {
            target_length: self.spool_length_task.target_length().get::<meter>(),
            on_completed_action: self.on_spool_length_task_complete,
        }
    }
}
