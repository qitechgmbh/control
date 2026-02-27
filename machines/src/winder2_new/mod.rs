use crate::LiveValues;
use crate::winder2_new::devices::Traverse;

mod base;
use std::time::Instant;

use base::MachineBase;

mod types;
use types::Mode;
use types::Hardware;

mod devices;
use devices::Puller;
use devices::TensionArm;
use devices::Laser;
use devices::Spool;

mod new;
mod act;
mod api;

mod machine_base
{
    pub use super::Winder2 as MachineImpl;
    pub use super::Namespace as Namespace;

    pub const MACHINE_ID:      u16   = crate::MACHINE_WINDER_V1;
    pub const MAX_CONNECTIONS: usize = 2;
    pub const MAX_MESSAGES:    u32   = 5;
}

#[derive(Debug)]
pub struct Winder2
{
    base: MachineBase,

    // machine config
    mode: Mode,

    // devices
    // --------------------------
    spool:       Spool,
    puller:      Puller,
    traverse:    Traverse,
    tension_arm: TensionArm,
    laser:       Laser,
}

impl Winder2
{
    fn new(base: MachineBase, hardware: Hardware) -> Self
    {
        Self { 
            base, 
            mode:        Mode::Standby, 
            spool:       Spool::new(hardware.spool_motor), 
            puller:      Puller::new(hardware.puller_motor), 
            traverse:    Traverse::new(hardware.traverse_motor, hardware.traverse_limit_switch),
            tension_arm: TensionArm::new(hardware.tension_arm_sensor), 
            laser:       Laser::new(hardware.laser),
        }
    }

    fn update(&mut self, now: Instant)
    {
        self.spool.update(now, &self.tension_arm, &self.puller);
        self.puller.update(now);
        self.traverse.update(&self.spool);

        if self.traverse.consume_state_changed() {
            // self.emit_state();
        }
    }

    fn receive_live_values(live_values: LiveValues)
    {
        use crate::LiveValues::Laser;

        #[allow(irrefutable_let_patterns)]
        if let Laser(live_values) = live_values 
        {
            _ = live_values;
            // TODO: use data to regulate speed. But not idea how, when what?
        }
    }
}