use std::time::{Duration, Instant};

use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::{digital_input::DigitalInput, digital_output::DigitalOutput, stepper_velocity_el70x1::StepperVelocityEL70x1};
use units::{Length, Velocity, length::{meter, millimeter}, velocity::millimeter_per_second};

use crate::{
    MachineChannel, 
    MachineWithChannel, 
    MachinesData, 
    VENDOR_QITECH, 
    machine_identification::{
        MachineIdentification, 
        MachineIdentificationUnique
    }, 
};

mod types;
use types::{Mode, Hardware, SpoolLengthTask, SpoolLengthTaskCompletedAction};

mod devices;
use devices::{Spool, traverse, Traverse, Puller, TensionArm};

mod api;
use api::{LiveValues, State};

mod new;

#[derive(Debug)]
pub struct Winder
{
    // common machine fields
    channel:   MachineChannel,
    last_emit: Instant,
    emitted_default_state: bool,

    // machine config
    mode: Mode,
    on_spool_length_task_complete: SpoolLengthTaskCompletedAction,

    // devices
    spool:       Spool,
    puller:      Puller,
    traverse:    Traverse,
    tension_arm: TensionArm,

    // tasks
    spool_length_task: SpoolLengthTask,

    // reference machine for the pullers adaptive speed mode
    puller_reference_machine: Option<MachineIdentificationUnique>,
}

// constants
impl Winder
{
    pub fn traverse_config() -> traverse::Config
    {
        use millimeter_per_second as mmps;
        use millimeter as mm;

        let speed_config = traverse::SpeedConfig {
            move_close:                          Velocity::new::<mmps>(10.0),
            move_not_close:                      Velocity::new::<mmps>(100.0),
            homing_escape_end_stop:              Velocity::new::<mmps>(10.0),
            homing_find_endstop_fine_distancing: Velocity::new::<mmps>(2.0),
            homing_find_endstop_coarse:          Velocity::new::<mmps>(-100.0),
            homing_find_endstop_fine:            Velocity::new::<mmps>(-2.0),
            traverse_going_out:                  Velocity::new::<mmps>(100.0),
        };

        traverse::Config {
            circumference:        Length::new::<mm>(35.0),
            steps_per_revolution: 200,
            micro_steps_per_step: 64,
            length_tolerance:     Length::new::<mm>(0.01),
            // defaults
            limit_inner_default:  Length::new::<mm>(22.0),
            limit_outer_default:  Length::new::<mm>(92.0),
            padding_default:      Length::new::<mm>(0.88),
            step_size_default:    Length::new::<mm>(1.75),

            validation_delay: Duration::from_millis(100),

            speed_config,
        }
    }
}

// public interface
impl Winder
{
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification 
    {
        vendor: VENDOR_QITECH,
        machine: crate::MACHINE_WINDER_V1,
    };

    pub fn new(channel: MachineChannel, hardware: Hardware) -> Self
    {
        let traverse = Traverse::new(
            Self::traverse_config(),
            hardware.traverse_motor, 
            hardware.traverse_limit_switch,
            hardware.traverse_laser_pointer,
        );

        Self { 
            channel, 
            last_emit:   Instant::now(),
            emitted_default_state: false,

            mode:        Mode::Standby, 
            spool:       Spool::new(hardware.spool_motor), 
            puller:      Puller::new(hardware.puller_motor), 
            traverse,
            tension_arm: TensionArm::new(hardware.tension_arm_sensor), 
            spool_length_task: SpoolLengthTask::new(),
            on_spool_length_task_complete: SpoolLengthTaskCompletedAction::NoAction,
            puller_reference_machine: None,
        }
    }

    pub fn can_wind(&self) -> bool 
    {
        let traverse_state = self.traverse.state();

        // Ensure both devices are calibrated
        self.tension_arm.is_zeroed() 
            && traverse_state.is_homed() 
            && !traverse_state.is_going_home()
    }
}

// base machine trait
impl MachineWithChannel for Winder
{
    type State = State;
    type LiveValues = LiveValues;

    fn update(&mut self, now: Instant) -> anyhow::Result<()>
    {
        let prev_traverse_state = self.traverse.state();

        self.spool.update(now, &self.tension_arm, &self.puller);
        self.puller.update(now);
        self.traverse.update(&self.spool);

        // update this last since it can mutate mode
        self.update_spool_length_task(now);

        // check if traverse state changed
        if prev_traverse_state != self.traverse.state()
        {
            self.emit_state();
        }

        if now.duration_since(self.last_emit) > Duration::from_secs_f64(1.0 / 30.0) 
        {
            self.emit_live_values();
        }

        Ok(())
    }

    fn receive_machines_data(&mut self, data: &MachinesData) 
    {
        use MachinesData::*;

        debug_assert!(self.puller_reference_machine.is_some());

        match data
        {
            Laser(state, live_values) => 
            {
                let current = live_values.diameter;
                let target  = state.laser_state.target_diameter;
                let lower   = state.laser_state.lower_tolerance;
                let upper   = state.laser_state.higher_tolerance;
                let modulation = Self::compute_modulation(current, target, lower, upper);

                let algorithm = &mut self.puller.speed_controller_algorithms_mut().adaptive;
                algorithm.set_modulation(modulation);
            },
            None => tracing::error!("Received MachineData::None"),
        };
    }

    fn subscribed_to_machine(&mut self, uid: MachineIdentificationUnique)
    {
        self.puller_reference_machine = Some(uid);
        self.emit_state();
    }

    fn unsubscribed_from_machine(&mut self, uid: MachineIdentificationUnique) 
    {
        if let Some(current_uid) = self.puller_reference_machine
        {
            if current_uid == uid
            {
                self.puller_reference_machine = None;
            }
        }
        
        self.emit_state();
    }

    fn get_state(&self) -> State 
    {
        self.create_state()
    }

    fn get_live_values(&self) -> Option<LiveValues> 
    {
        Some(self.create_live_values())
    }

    fn mutate(&mut self, value: serde_json::Value) -> anyhow::Result<()> 
    {
        self.handle_mutation(value)
    }

    fn on_namespace(&mut self) 
    {
        self.emit_state();
    }

    fn get_machine_channel(&self) -> &MachineChannel 
    {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel 
    {
        &mut self.channel
    }
}

// utils
impl Winder
{
    fn compute_modulation(current: f64, target: f64, lower: f64, upper: f64) -> f64 
    {
        let lower_bound = target - lower;
        let upper_bound = target + upper;

        if current <= lower_bound { return -1.0 };
        if current >= upper_bound { return 1.0 };

        // 
        if current < target {
            let min = lower_bound;
            let max = target;
            -((current - min) / (max - min))
        } else {
            let min = target;
            let max = upper_bound;
            (current - min) / (max - min)
        }
    }

    /// Updates the spool length task and changes the 
    /// machines mode as appropiate
    fn update_spool_length_task(&mut self, now: Instant)
    {
        // TODO(JSE): find out if update_timer can be replaced 
        // with update_progress(), since speed should be 0

        // refactor of: stop_or_pull_spool
        use SpoolLengthTaskCompletedAction::*;

        let velocity = self.puller.output_speed();

        // always update progress if no action
        if self.on_spool_length_task_complete == NoAction
        {
            self.spool_length_task.update_progress(now, velocity);
            return;
        }

        // don't update progress if no mode to move the puller
        // is active... 
        //
        // jse: But why even bother, if it ain't moving anyway?
        // if the motor doesn't stop immediately won't we have
        // a false value then since we stop counting while
        // it slows down??
        if self.mode != Mode::Pull && self.mode != Mode::Wind
        {
            self.spool_length_task.update_timer(now);
            return;
        }

        self.spool_length_task.update_progress(now, velocity);

        if self.spool_length_task.is_complete()
        {
            let mode = match self.on_spool_length_task_complete
            {
                Pull => Mode::Pull,
                Hold => Mode::Hold,
                NoAction => return, // unreachable
            };

            self.spool_length_task.reset(now);

            // can only fail if attempting to set mode to Wind so can't fail here
            _ = self.set_mode(mode);
        }
    }

    fn emit_state(&mut self)
    {
        let event = self.get_state();
        self.emitted_default_state = true;
        self.channel.emit(event);
    }

    fn emit_live_values(&mut self)
    {
        let event = self.create_live_values();
        self.channel.emit(event);
        self.last_emit = Instant::now();
    }
}