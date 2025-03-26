use crate::machines::{winder1::WinderV1, MACHINE_WINDER_V1, VENDOR_QITECH};
use control_core::{identification::MachineIdentification, machines::registry::MachineRegistry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<WinderV1>(MachineIdentification::new(VENDOR_QITECH, MACHINE_WINDER_V1));
        mc
    };
}
