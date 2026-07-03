const POOL_CAPACITY_I64: usize = 1024;
const POOL_CAPACITY_F64: usize = 1024;

mod properties;
pub use properties::{
    IntProperty,
    FloatProperty,
    BoolProperty,
    EnumProperty,
    LengthProperty,
};

mod pool;
use pool::PropertyPool;
pub type IntPool = PropertyPool<i64, POOL_CAPACITY_I64>;
pub type FloatPool = PropertyPool<f64, POOL_CAPACITY_F64>;

mod allocator;
pub use allocator::PropertyAllocator;
pub use allocator::PropertyAllocatorError;

mod export;

pub use export::{
    PropertyBatch,
    PropertyBatchExporter
};
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PropertyEntry<T> {
    pub ident: u64,
    pub name: String,
    pub value: T,
}

impl<T: Default> PropertyEntry<T> {
    pub fn new(ident: u64, name: String, value: T) -> Self {
        Self { ident, name, value }
    }
}
