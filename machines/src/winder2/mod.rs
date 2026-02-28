use std::time::{Duration, Instant};

use units::{angle::degree, angular_velocity::revolution_per_minute, length::{meter, millimeter}, velocity::meter_per_minute};

use crate::{
    MachineChannel, 
    MachineWithChannel, 
    MachineConnection, 
    MachinesLiveValues, 
    VENDOR_QITECH,
    machine_identification::MachineIdentification
};

mod types;
use types::{Mode, Hardware, SpoolLengthTaskCompletedAction};

mod tasks;
use tasks::SpoolLengthTask;

mod devices;
use devices::{Spool, Traverse, Puller, TensionArm};

mod api;
use api::{LiveValues, State};

mod new;
mod mutation;
mod events;


#[derive(Debug)]
pub struct Winder2
{
    // common machine fields
    channel:   MachineChannel,
    last_emit: Instant,

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
    puller_speed_reference_machine: Option<MachineConnection>,
}

impl MachineWithChannel for Winder2
{
    type State = State;
    type LiveValues = LiveValues;

    fn update(&mut self, now: std::time::Instant) -> anyhow::Result<()> 
    {
        self.spool.update(now, &self.tension_arm, &self.puller);
        self.puller.update(now);

        let state_changed = self.traverse.update(&self.spool);

        // update last since it can mutate mode
        self.update_spool_length_task(now);

        if state_changed
        {
            self.emit_state();
        }

        if now.duration_since(self.last_emit) > Duration::from_secs_f64(1.0 / 30.0) 
        {
            self.emit_live_values();
        }

        Ok(())
    }

    fn mutate(&mut self, value: serde_json::Value) -> anyhow::Result<()> 
    {
        let mutation: Mutation = serde_json::from_value(value)?;

        todo!("apply mutation");

        Ok(())
    }

    fn get_live_values(&self) -> Option<LiveValues> 
    {
        let tension_arm_angle = self.tension_arm.get_angle().get::<degree>();

        // Wrap [270;<360] to [-90; 0]
        // This is done to reduce flicker in the graphs around the zero point
        let tension_arm_angle = match tension_arm_angle >= 270.0
        {
            true  => tension_arm_angle - 360.0,
            false => tension_arm_angle,
        };

        Some(LiveValues {
            traverse_position: self.traverse.current_position().map(|x| x.get::<millimeter>()),
            puller_speed: self.puller.motor_speed().get::<meter_per_minute>(),
            spool_rpm: self.spool.speed().get::<revolution_per_minute>(),
            spool_progress: self.spool_length_task.current_length().get::<meter>(),
            tension_arm_angle,
        })
    }

    fn get_state(&self) -> State 
    {
        todo!()
    }

    fn on_namespace(&mut self) 
    {
        self.emit_state();
    }

    fn on_receive_live_values(&mut self, live_values: MachinesLiveValues) 
    {
        use crate::MachinesLiveValues::Laser;

        #[allow(irrefutable_let_patterns)]
        if let Laser(live_values) = live_values 
        {
            _ = live_values;
            // TODO: use data to regulate speed. But not idea how, when what?
        }
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

impl Winder2
{
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification 
    {
        vendor: VENDOR_QITECH,
        machine: crate::MACHINE_WINDER_V1,
    };

    fn new(channel: MachineChannel, hardware: Hardware) -> Self
    {
        let traverse = Traverse::new(
            hardware.traverse_motor, 
            hardware.traverse_limit_switch,
            hardware.traverse_laser_pointer,
        );

        Self { 
            channel, 
            last_emit:   Instant::now(),
            mode:        Mode::Standby, 
            spool:       Spool::new(hardware.spool_motor), 
            puller:      Puller::new(hardware.puller_motor), 
            traverse,
            tension_arm: TensionArm::new(hardware.tension_arm_sensor), 
            spool_length_task: SpoolLengthTask::new(),
            on_spool_length_task_complete: SpoolLengthTaskCompletedAction::NoAction,
            puller_speed_reference_machine: None,
        }
    }

    fn emit_state(&mut self) 
    {
        let event = self.get_state();
        self.channel.emit(event);
    }

    fn emit_live_values(&mut self) 
    {
        let event = self.get_live_values();
        self.channel.emit(event);
        self.last_emit = Instant::now();
    }
}

// utils
impl Winder2
{
    /// Can wind capability check
    pub fn can_wind(&self) -> bool 
    {
        // Check if tension arm is calibrated and traverse is homed
        self.tension_arm.is_calibrated() 
            && self.traverse.is_homed() 
            && !self.traverse.is_going_home()

        // TODO(JSE): why the 2 checks for traverse?
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

        let mode = match self.on_spool_length_task_complete
        {
            Pull => Mode::Pull,
            Hold => Mode::Hold,
            NoAction => return, // unreachable
        };

        self.spool_length_task.reset(now);
        self.set_mode(mode);
    }
}