use crate::machines::{MACHINE_WINDER_V1, VENDOR_QITECH, winder2::Winder2};
use control_core::machines::{identification::MachineIdentification, registry::MachineRegistry};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<Winder2>(MachineIdentification::new(VENDOR_QITECH, MACHINE_WINDER_V1));
        mc
    };
}
