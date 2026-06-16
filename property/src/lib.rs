const POOL_CAPACITY_MAX_FLOAT: usize = 1024;
const POOL_CAPACITY_MAX_INTEGER: usize = 1024;
const POOL_CAPACITY_MAX_BOOLEAN: usize = 256;
const POOL_CAPACITY_MAX_STRING: usize = 64;

mod property;
use std::fmt::Debug;

pub use property::IntProperty;
pub use property::FloatProperty;
pub use property::BoolProperty;
pub use property::EnumProperty;
pub use property::StringProperty;
pub use property::LengthProperty;

mod pool;
pub use pool::PropertyPool;

mod set;
pub use set::PropertySet;

mod allocator;
pub use allocator::Allocator;
pub use allocator::AllocatorError;

#[cfg(feature = "serde")]
mod codec;

#[cfg(feature = "serde")]
pub use codec::{
    DirtyPropertyPoolExportView,
    DirtyPropertySetExportView,
    ExportedPropertySet,
};

pub type StringPropertyValue = heapless::String<128>;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Default, Clone)]
pub struct PropertyEntry<T: Default + Debug> {
    pub ident: u64,
    pub name: &'static str,
    pub value: T,
}

impl<T: Default + Debug> PropertyEntry<T> {
    pub fn new(ident: u64, name: &'static str, value: T) -> Self {
        Self { ident, name, value }
    }
}
