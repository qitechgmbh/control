const POOL_CAPACITY_MAX_FLOAT: usize = 1024;
const POOL_CAPACITY_MAX_INTEGER: usize = 1024;

mod property;
pub use property::IntProperty;
pub use property::FloatProperty;
pub use property::BoolProperty;
pub use property::EnumProperty;
pub use property::LengthProperty;

mod pool;
pub use pool::PropertyPool;

mod set;
pub use set::PropertySet;

mod allocator;
pub use allocator::Allocator;
pub use allocator::AllocatorError;

mod view;
pub use view::PropertySetView;
pub use view::PropertyView;
pub use view::PropertyBufferIter;

mod export;
pub use export::ExportedPropertyEntry;
pub use export::ExportedPropertySet;

#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[derive(Debug, Default, Clone)]
pub struct PropertyEntry<T> {
    pub ident: u64,
    pub name: &'static str,
    pub value: T,
}

impl<T: Default> PropertyEntry<T> {
    pub fn new(ident: u64, name: &'static str, value: T) -> Self {
        Self { ident, name, value }
    }
}
