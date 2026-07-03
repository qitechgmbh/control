use std::any::TypeId;
use std::collections::HashMap;

const MAX_DATA_LEN: usize = 2048;

#[cfg(feature = "schema")]
mod schema;

#[cfg(feature = "schema")]
pub use schema::{
    MachineSchema,
    // MutationParameterType,
    PropertySchemaNode,
    PropertySchema,
};

mod hardware;
pub use hardware::Hardware;
pub use hardware::MachineHardware;

pub mod property;

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MachineIdentification {
    pub vendor_id: u16,
    pub machine_id: u16,
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct MachineIdentificationUnique {
    pub ident: MachineIdentification,
    pub serial: u32,
}

impl MachineIdentificationUnique {
    pub fn as_u64(&self) -> u64 {
        // Pack 16 + 16 + 32 bits into one 64-bit integer
        ((self.ident.vendor_id as u64) << 48)
            | ((self.ident.machine_id as u64) << 32)
            | (self.serial as u64)
    }

    pub fn from_u64(value: u64) -> Self {
        Self {
            ident: MachineIdentification {
                vendor_id: (value >> 48) as u16,
                machine_id: (value >> 32) as u16,
            },
            serial: value as u32,
        }
    }
}

pub trait Machine {
    fn act(&mut self, machine_data: Option<&mut MachineDataRegistry>) -> Result<(), MachineError>;
    fn react(&mut self, registry: &MachineDataRegistry);
}

pub trait MachineNew: Sized {
    fn new(args: MachineNewArgs) -> anyhow::Result<Self>;
}

pub struct MachineNewArgs<'a> {
    pub ident: MachineIdentificationUnique,
    pub hardware: MachineHardware,
    pub properties: property::PropertyAllocator<'a>,
}

#[repr(align(64))]
pub struct MachineData {
    pub type_id: TypeId,
    pub length: usize,
    pub data: [u8; MAX_DATA_LEN],
}

pub struct MachineDataRegistry {
    // Each slot is a fixed-size buffer
    pub storage: HashMap<MachineIdentificationUnique, MachineData>,
}

impl MachineDataRegistry {
    pub fn zero_entry(&mut self, ident: MachineIdentificationUnique) {
        if let Some(m) = self.storage.get_mut(&ident) {
            m.length = 0;
            m.data.fill(0);
        }
    }
}

#[derive(Debug, Clone)]
pub enum MachineError {
    RecoverableFailure(String),
    IrrecoverableFailure(String),
}