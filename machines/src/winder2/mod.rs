use std::time::{Duration, Instant}
;
use crate::winder2::api::TraverseState;
use crate::winder2::devices::TraverseMode;
use crate::{MachinesLiveValues, VENDOR_QITECH};
use crate::machine_identification::MachineIdentification;
use crate::{MachineChannel, MachineWithChannel};
use devices::Traverse;

mod types;
use types::Mode;
use types::Hardware;

mod devices;
use devices::Puller;
use devices::TensionArm;
use devices::LaserPointer;
use devices::Spool;

mod new;
mod mutation;

mod api;
pub use api::LiveValues;
pub use api::State;

mod events;

mod automatic_action;
use automatic_action::AutomaticAction;
use automatic_action::Mode as AutomaticActionMode;
use units::angle::degree;

#[derive(Debug)]
pub struct Winder2
{
    channel:   MachineChannel,
    last_emit: Instant,

    // machine config
    mode: Mode,

    // devices
    spool:       Spool,
    puller:      Puller,
    traverse:    Traverse,
    tension_arm: TensionArm,

    // actions
    automatic_action: AutomaticAction
}

impl MachineWithChannel for Winder2
{
    type State = State;
    type LiveValues = LiveValues;

    fn get_machine_channel(&self) -> &MachineChannel 
    {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel 
    {
        &mut self.channel
    }

    fn update(&mut self, now: std::time::Instant) -> anyhow::Result<()> 
    {
        self.spool.update(now, &self.tension_arm, &self.puller);
        self.puller.update(now);
        self.traverse.update(&self.spool);

        if let Some(next_mode) = self.automatic_action.update(now, self.mode, &self.puller)
        {
            self.set_mode(next_mode);
        }

        if self.traverse.consume_state_changed() 
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



        None
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
            hardware.traverse_laser,
            hardware.traverse_limit_switch
        );

        Self { 
            channel, 
            last_emit:   Instant::now(),
            mode:        Mode::Standby, 
            spool:       Spool::new(hardware.spool_motor), 
            puller:      Puller::new(hardware.puller_motor), 
            traverse,
            tension_arm: TensionArm::new(hardware.tension_arm_sensor), 
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

        // TODO: why the 2 checks for traverse?
    }

    
}