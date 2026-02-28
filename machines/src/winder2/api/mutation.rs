use serde::{Deserialize, Serialize};
use units::{
    AngularVelocity, 
    Length, 
    Velocity, 
    angular_velocity::revolution_per_minute, 
    length::{meter, millimeter}, 
    velocity::meter_per_minute
};

use crate::{
    machine_identification::MachineIdentificationUnique, 
    types::Direction,
    winder2::{Winder2, devices::{OperationState, PullerGearRatio, SpoolSpeedControlMode}, types::{Mode, SpoolLengthTaskCompletedAction}}};

#[derive(Deserialize, Serialize)]
pub enum Mutation 
{
    // Traverse
    /// Position in mm from home point
    SetTraverseLimitOuter(f64),
    /// Position in mm from home point
    SetTraverseLimitInner(f64),
    /// Step size in mm for traverse movement
    SetTraverseStepSize(f64),
    /// Padding in mm for traverse movement limits
    SetTraversePadding(f64),
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    /// Find home point
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),

    // Puller
    /// on = speed, off = stop
    // SetPullerRegulationMode(PullerRegulationMode), TODO: replace
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
    SetPullerGearRatio(PullerGearRatio),

    // Spool Speed Controller
    SetSpoolRegulationMode(SpoolSpeedControlMode),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),
    SetSpoolForward(bool),

    // Adaptive Spool Speed Controller Parameters
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Spool Auto Stop/Pull
    SetSpoolAutomaticRequiredMeters(f64),
    SetSpoolAutomaticAction(SpoolLengthTaskCompletedAction),
    ResetSpoolProgress,

    // Tension Arm
    ZeroTensionArmAngle,

    // Mode
    SetMode(Mode),

    // Connected Machine
    SetConnectedMachine(MachineIdentificationUnique),

    // Disconnect Machine
    DisconnectMachine(MachineIdentificationUnique),
}

impl Winder2
{
    pub fn handle_mutation(&mut self, value: serde_json::Value) -> Result<(), anyhow::Error> 
    {
        use Mutation::*;

        let mutation: Mutation = serde_json::from_value(value)?;

        match mutation
        {
            // machine
            SetMode(v) => self.set_mode(v),

            //traverse
            SetTraverseLimitOuter(v)      => self.traverse_set_limit_outer(v),
            SetTraverseLimitInner(v)      => self.traverse_set_limit_inner(v),
            SetTraverseStepSize(v)        => self.traverse_set_step_size(v),
            SetTraversePadding(v)         => self.traverse_set_padding(v),
            GotoTraverseLimitOuter        => self.traverse_goto_limit_outer(),
            GotoTraverseLimitInner        => self.traverse_goto_limit_inner(),
            GotoTraverseHome              => self.traverse_goto_home(),
            EnableTraverseLaserpointer(v) => self.traverse_set_laser_pointer_enabled(v),

            // puller
            // SetPullerRegulationMode(_) => todo!(),
            SetConnectedMachine(_) => todo!(), //TODO: change to REFERENCE
            DisconnectMachine(_)   => todo!(), //TODO: change to REFERENCE
            SetPullerTargetSpeed(v)    => self.puller_set_target_speed(v),
            SetPullerTargetDiameter(v) => self.puller_set_target_diameter(v),
            SetPullerForward(v)        => self.puller_set_direction(Direction::from_bool(v)),
            SetPullerGearRatio(v)      => self.puller_set_gear_ratio(v),

            // spool
            SetSpoolRegulationMode(_) => todo!(),
            SetSpoolMinMaxMinSpeed(_) => todo!(),
            SetSpoolMinMaxMaxSpeed(_) => todo!(),
            SetSpoolForward(_) => todo!(),
            SetSpoolAdaptiveTensionTarget(_) => todo!(),
            SetSpoolAdaptiveRadiusLearningRate(_) => todo!(),
            SetSpoolAdaptiveMaxSpeedMultiplier(_) => todo!(),
            SetSpoolAdaptiveAccelerationFactor(_) => todo!(),
            SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(_) => todo!(),

            // tension arm
            ZeroTensionArmAngle => todo!(),

            // spool length task
            SetSpoolAutomaticRequiredMeters(_) => todo!(),
            SetSpoolAutomaticAction(_) => todo!(),
            ResetSpoolProgress => todo!(),
        }

        Ok(())
    }
}

// Machine
impl Winder2 
{
    pub fn set_mode(&mut self, mode: Mode)
    {
        let should_update = mode != Mode::Wind || self.can_wind();

        if should_update
        {
            self.mode = mode;
            match self.mode
            {
                Mode::Standby =>
                {
                    let state = OperationState::Disabled;
                    self.spool.set_operation_state(state);
                    self.puller.set_device_state(state);
                    self.traverse.set_device_state(state);
                },
                Mode::Hold => 
                {
                    let state = OperationState::Holding;
                    self.spool.set_operation_state(state);
                    self.puller.set_device_state(state);
                    self.traverse.set_device_state(state);
                },
                Mode::Pull => 
                {
                    use OperationState::*;
                    self.spool.set_operation_state(Holding);
                    self.puller.set_device_state(Running);
                    self.traverse.set_device_state(Holding);
                },
                Mode::Wind => 
                {
                    let state = OperationState::Running;
                    self.spool.set_operation_state(state);
                    self.puller.set_device_state(state);
                    self.traverse.set_device_state(state);
                },
            }
        }

        self.emit_state();
    }
}

// Tension Arm (COMPLETE)
impl Winder2
{
    pub fn tension_arm_calibrate(&mut self)
    {
        self.tension_arm.calibrate();
        self.emit_live_values(); // For angle update
        self.emit_state();
    }
}

// Puller
impl Winder2
{
    // TODO: replace with connection logic
    // pub fn puller_set_regulation(&mut self, puller_regulation_mode: PullerRegulationMode) {
    //     self.puller_speed_controller
    //         .set_regulation_mode(puller_regulation_mode);
    //     self.emit_state();
    // }

    /// Set target speed in m/min
    pub fn puller_set_target_speed(&mut self, target_speed: f64) 
    {
        // Convert m/min to velocity
        let target_speed = Velocity::new::<meter_per_minute>(target_speed);
        self.puller.set_target_speed(target_speed);
        self.emit_state();
    }

    /// Set target diameter in mm
    pub fn puller_set_target_diameter(&mut self, target_diameter: f64) 
    {
        // Convert m/min to velocity
        let target_diameter = Length::new::<millimeter>(target_diameter);
        self.puller.set_target_diameter(target_diameter);
        self.emit_state();
    }

    /// Set direction
    pub fn puller_set_direction(&mut self, direction: Direction) 
    {
        self.puller.set_direction(direction);
        self.emit_state();
    }

    /// Set gear ratio for winding speed
    pub fn puller_set_gear_ratio(&mut self, gear_ratio: PullerGearRatio) 
    {
        self.puller.set_gear_ratio(gear_ratio);
        self.emit_state();
    }
}

// Spool
impl Winder2
{
    /// Set forward rotation direction
    pub fn spool_set_direction(&mut self, value: Direction) 
    {
        self.spool.set_direction(value);
        self.emit_state();
    }

    /// Set speed control mode
    pub fn spool_set_speed_control_mode(&mut self, value: SpoolSpeedControlMode)
    {
        self.spool.set_speed_control_mode(value);
        self.emit_state();
    }

    /// Set minimum speed for minmax mode in RPM
    pub fn spool_set_minmax_min_speed(&mut self, min_speed_rpm: f64) 
    {
        let min_speed = AngularVelocity::new::<revolution_per_minute>(min_speed_rpm);
        
        if let Err(e) = self.spool.speed_controllers.minmax.set_min_speed(min_speed)
        {
            tracing::error!("Failed to set spool min speed: {:?}", e);
        }

        self.emit_state();
    }

    /// Set maximum speed for minmax mode in RPM
    pub fn spool_set_minmax_max_speed(&mut self, max_speed_rpm: f64) 
    {
        let max_speed = AngularVelocity::new::<revolution_per_minute>(max_speed_rpm);

        if let Err(e) = self.spool.speed_controllers.minmax.set_max_speed(max_speed)
        {
            tracing::error!("Failed to set spool max speed: {:?}", e);
        }

        self.emit_state();
    }

    /// Set tension target for adaptive mode (0.0-1.0)
    pub fn spool_set_adaptive_tension_target(&mut self, tension_target: f64) 
    {
        self.spool.speed_controllers.adaptive.set_tension_target(tension_target);

        self.emit_state();
    }

    /// Set radius learning rate for adaptive mode
    pub fn spool_set_adaptive_radius_learning_rate(&mut self, radius_learning_rate: f64) {
        self.spool.speed_controllers.adaptive
            .set_radius_learning_rate(radius_learning_rate);
        self.emit_state();
    }

    /// Set max speed multiplier for adaptive mode
    pub fn spool_set_adaptive_max_speed_multiplier(&mut self, max_speed_multiplier: f64) {
        self.spool.speed_controllers.adaptive
            .set_max_speed_multiplier(max_speed_multiplier);
        self.emit_state();
    }

    /// Set acceleration factor for adaptive mode
    pub fn spool_set_adaptive_acceleration_factor(&mut self, acceleration_factor: f64) {
        self.spool.speed_controllers.adaptive
            .set_acceleration_factor(acceleration_factor);
        self.emit_state();
    }

    /// Set deacceleration urgency multiplier for adaptive mode
    pub fn spool_set_adaptive_deacceleration_urgency_multiplier(
        &mut self,
        deacceleration_urgency_multiplier: f64,
    ) {
        self.spool.speed_controllers.adaptive
            .set_deacceleration_urgency_multiplier(deacceleration_urgency_multiplier);
        self.emit_state();
    }
}

// Traverse
impl Winder2
{
    pub fn traverse_set_laser_pointer_enabled(&mut self, value: bool) 
    {
        self.traverse.set_laser_pointer_enabled(value);
        self.emit_state();
    }

    pub fn traverse_set_limit_inner(&mut self, limit: f64) 
    {
        let limit_inner = Length::new::<millimeter>(limit);
        _ = self.traverse.try_set_limit_inner(limit_inner);
        self.emit_state();
    }

    pub fn traverse_set_limit_outer(&mut self, limit: f64) 
    {
        let limit_outer = Length::new::<millimeter>(limit);
        _ = self.traverse.try_set_limit_outer(limit_outer);
        self.emit_state();
    }

    pub fn traverse_set_step_size(&mut self, step_size: f64) 
    {
        let step_size = Length::new::<millimeter>(step_size);
        self.traverse.set_step_size(step_size);
        self.emit_state();
    }

    pub fn traverse_set_padding(&mut self, padding: f64) 
    {
        let padding = Length::new::<millimeter>(padding);
        self.traverse.set_padding(padding);
        self.emit_state();
    }

    pub fn traverse_goto_limit_inner(&mut self) 
    {
        _ = self.traverse.try_goto_limit_inner();
        self.emit_state();
    }

    pub fn traverse_goto_limit_outer(&mut self) 
    {
        _ = self.traverse.try_goto_limit_outer();
        self.emit_state();
    }

    pub fn traverse_goto_home(&mut self) 
    {
        _ = self.traverse.try_goto_home();
        self.emit_state();
    }
}

// Spool Length Task
impl Winder2
{
    pub fn set_spool_length_task_target_length(&mut self, meters: f64) 
    {
        let target_length = Length::new::<meter>(meters);
        self.spool_length_task.set_target_length(target_length);
        self.emit_state();
    }

    pub fn set_on_spool_length_task_completed_action(
        &mut self, action: SpoolLengthTaskCompletedAction) 
    {
        self.on_spool_length_task_complete = action;
        self.emit_state();
    }
}