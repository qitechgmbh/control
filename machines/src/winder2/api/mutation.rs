use std::time::Instant;

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
    AsyncThreadMessage, CrossConnection, Machine, machine_identification::MachineIdentificationUnique, types::Direction, winder2::{Winder2, devices::{
        OperationState, 
        PullerGearRatio, 
        PullerSpeedControlMode,
        SpoolSpeedControlMode
    }, types::{Mode, SpoolLengthTaskCompletedAction}}
};

#[derive(Deserialize, Serialize)]
pub enum Mutation 
{
    // Machine
    SetMode(Mode),

    // Spool
    SetSpoolDirection(Direction),
    SetSpoolSpeedControlMode(SpoolSpeedControlMode),

    // Spool Speed MinMax Strategy
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),

    // Spool Speed Adaptive Strategy
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),

    // Puller
    SetPullerDirection(Direction),
    SetPullerGearRatio(PullerGearRatio),
    SetPullerSpeedControlMode(PullerSpeedControlMode),

    // Puller Speed Fixed Strategy
    SetPullerFixedTargetSpeed(f64),

    // Puller Speed Strategy
    SetPullerAdaptiveReferenceMachine(Option<MachineIdentificationUnique>),
    SetPullerAdaptiveBaseSpeed(f64),  // in m/min
    SetPullerAdaptiveDeviationMax(f64), // in m/min

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

    // Tension Arm
    ZeroTensionArmAngle,

    // Spool Length Task
    SetSpoolLengthTaskTargetLength(f64),
    ResetSetSpoolLengthTaskProgress,
    SetOnSpoolLengthTaskCompletedAction(SpoolLengthTaskCompletedAction),
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

            // spool
            SetSpoolDirection(v) => self.spool_set_direction(v),
            SetSpoolSpeedControlMode(v) => self.spool_set_speed_control_mode(v),
            SetSpoolMinMaxMinSpeed(v) => self.spool_set_minmax_min_speed(v),
            SetSpoolMinMaxMaxSpeed(v) => self.spool_set_minmax_max_speed(v),
            SetSpoolAdaptiveTensionTarget(v) => self.spool_set_adaptive_tension_target(v),
            SetSpoolAdaptiveRadiusLearningRate(v) => self.spool_set_adaptive_radius_learning_rate(v),
            SetSpoolAdaptiveMaxSpeedMultiplier(v) => self.spool_set_adaptive_max_speed_multiplier(v),
            SetSpoolAdaptiveAccelerationFactor(v) => self.spool_set_adaptive_acceleration_factor(v),
            SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(v) => self.spool_set_adaptive_deacceleration_urgency_multiplier(v),

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
            SetPullerDirection(v)                => self.puller_set_direction(v),
            SetPullerGearRatio(v)                => self.puller_set_gear_ratio(v),
            SetPullerSpeedControlMode(v)         => self.puller_set_speed_control_mode(v),
            SetPullerFixedTargetSpeed(v)         => self.puller_set_fixed_target_speed(v),
            SetPullerAdaptiveReferenceMachine(v) => self.puller_set_adaptive_reference_machine(v)?,
            SetPullerAdaptiveBaseSpeed(v)        => self.puller_set_adaptive_base_speed(v),
            SetPullerAdaptiveDeviationMax(v)     => self.puller_set_adaptive_deviation_max(v),

            // tension arm
            ZeroTensionArmAngle => self.tension_arm_calibrate(),

            // spool length task
            SetSpoolLengthTaskTargetLength(v) => 
                self.spool_length_task_set_target_length(v),
            ResetSetSpoolLengthTaskProgress => 
                self.spool_length_task_reset(Instant::now()),
            SetOnSpoolLengthTaskCompletedAction(v) => 
                self.set_on_spool_length_task_complete(v),
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
                    self.puller.set_operation_state(state);
                    self.traverse.set_device_state(state);
                },
                Mode::Hold => 
                {
                    let state = OperationState::Holding;
                    self.spool.set_operation_state(state);
                    self.puller.set_operation_state(state);
                    self.traverse.set_device_state(state);
                },
                Mode::Pull => 
                {
                    use OperationState::*;
                    self.spool.set_operation_state(Holding);
                    self.puller.set_operation_state(Running);
                    self.traverse.set_device_state(Holding);
                },
                Mode::Wind => 
                {
                    let state = OperationState::Running;
                    self.spool.set_operation_state(state);
                    self.puller.set_operation_state(state);
                    self.traverse.set_device_state(state);
                },
            }
        }

        self.emit_state();
    }
}

// Tension Arm
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
    pub fn puller_set_speed_control_mode(&mut self, mode: PullerSpeedControlMode) 
    {
        self.puller.speed_control_set_mode(mode);
        self.emit_state();
    }

    pub fn puller_set_direction(&mut self, direction: Direction) 
    {
        self.puller.set_direction(direction);
        self.emit_state();
    }

    pub fn puller_set_gear_ratio(&mut self, gear_ratio: PullerGearRatio) 
    {
        self.puller.set_gear_ratio(gear_ratio);
        self.emit_state();
    }

    /// target_speed in m/min
    pub fn puller_set_fixed_target_speed(&mut self, speed: f64) 
    {
        // Convert m/min to velocity
        let speed = Velocity::new::<meter_per_minute>(speed);
        self.puller.speed_controller_strategies_mut().fixed.set_target_speed(speed);
        self.emit_state();
    }

    pub fn puller_set_adaptive_base_speed(&mut self, speed: f64) 
    {
        let speed = Velocity::new::<meter_per_minute>(speed);
        self.puller.speed_controller_strategies_mut().adaptive.set_base_speed(speed);
        self.emit_state();
    }

    pub fn puller_set_adaptive_deviation_max(&mut self, deviation_max: f64) 
    {
        let speed = Velocity::new::<meter_per_minute>(deviation_max);
        self.puller.speed_controller_strategies_mut().adaptive.set_deviation_max(speed);
        self.emit_state();
    }

    pub fn puller_set_adaptive_reference_machine(
        &mut self, 
        machine_uid: Option<MachineIdentificationUnique>
    ) -> Result<(), anyhow::Error>
    {
        match machine_uid
        {
            Some(machine_uid) => 
            {
                if let Some(connection) = &self.puller_speed_reference_machine
                {
                    if connection.ident == machine_uid 
                        { return Ok(()); }
                }

                let main_sender = match &self.channel.main_sender 
                {
                    Some(v) => v,
                    None => 
                    {
                        return Err(anyhow::anyhow!(
                            "{:?} Failed to connect to {:?}",
                            self.get_machine_identification_unique(),
                            machine_uid,
                        ));
                    }
                };

                main_sender.try_send(AsyncThreadMessage::ConnectTwoWayRequest(
                    CrossConnection {
                        src: self.get_machine_identification_unique(),
                        dest: machine_uid,
                    },
                ))?;

                self.emit_state();
            },
            None => 
            {
                match self.puller_speed_reference_machine.take()
                {
                    Some(connection) => 
                    {
                        _ = connection;
                        //TODO: Tell other machine to disconnect

                        let main_sender = match &self.channel.main_sender 
                        {
                            Some(v) => v,
                            None => 
                            {
                                return Err(anyhow::anyhow!(
                                    "{:?} Failed to connect to {:?}",
                                    self.get_machine_identification_unique(),
                                    machine_uid,
                                ));
                            }
                        };

                        main_sender.try_send(AsyncThreadMessage::DisconnectMachines(
                            CrossConnection {
                                src:  self.get_machine_identification_unique(),
                                dest: connection.ident,
                            },
                        ))?;
                    },
                    None => return Ok(()), // nothing to do
                }
            },
        }

        self.emit_state();
        Ok(())
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
    pub fn spool_length_task_set_target_length(&mut self, meters: f64) 
    {
        let target_length = Length::new::<meter>(meters);
        self.spool_length_task.set_target_length(target_length);
        self.emit_state();
    }

    pub fn spool_length_task_reset(&mut self, now: Instant) 
    {
        self.spool_length_task.reset(now);
        self.emit_state();
    }

    pub fn set_on_spool_length_task_complete(&mut self, action: SpoolLengthTaskCompletedAction) 
    {
        self.on_spool_length_task_complete = action;
        self.emit_state();
    }
}