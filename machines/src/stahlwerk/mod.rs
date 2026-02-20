use crate::{
    VENDOR_QITECH, 
    machine_identification::MachineIdentification, 
    registry::MachineRegistry, 
    stahlwerk::{ machine_id::MachineId } 
};

pub mod r#abstract;
pub mod template;
pub mod machine_id;

pub mod ff01_mock;

pub const CATEGORY_ID: u8 = 0x02; // stahlwerk category

pub const FF01_MOCK: MachineId = MachineId::new(CATEGORY_ID, 0, "stahlwerk_ff01");

pub const fn get_slug(id: u16) -> Option<&'static str> 
{
    match id 
    {
        id if id == FF01_MOCK.to_u16() => Some(FF01_MOCK.slug),
        _ => None,
    }
}

pub fn register_machines(registry: &mut MachineRegistry)
{
    #[cfg(feature = "mock-machine")]
    registry.register::<FF01MachineMock>(new_identification(FF01_MOCK));
}

fn new_identification(id: MachineId) -> MachineIdentification
{
    MachineIdentification {
        vendor:  VENDOR_QITECH,
        machine: id.to_u16(),
    }
}