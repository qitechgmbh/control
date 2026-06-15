use qitech_lib::machines::MachineIdentificationUnique;

pub(super) const STRING_LEN_MAX: usize = 128;
pub(super) const POOL_CAPACITY_MAX_FLOAT: usize = 1024;
pub(super) const POOL_CAPACITY_MAX_INTEGER: usize = 1024;
pub(super) const POOL_CAPACITY_MAX_BOOLEAN: usize = 1024;
pub(super) const POOL_CAPACITY_MAX_STRING: usize = 64;

mod base;
use base::SimpleProperty;
pub use base::EnumProperty;
pub use base::StringProperty;

mod uom;
pub use uom::LengthProperty;

mod iterator;
pub use iterator::Iter;
pub use iterator::IterItem;

mod pool;
pub use pool::Pool;

mod allocator;
pub use allocator::Allocator;
pub use allocator::AllocatorError;

pub type BoolProperty = SimpleProperty<bool>;
pub type FloatProperty = SimpleProperty<f64>;
pub type IntProperty = SimpleProperty<i64>;

#[derive(Debug, Clone)]
pub(super) struct PropertyEntry<T> {
    pub ident: MachineIdentificationUnique,
    pub name: &'static str,
    pub value: T,
    pub dirty: bool,
}

impl<T> PropertyEntry<T> {
    fn new(ident: MachineIdentificationUnique, name: &'static str, value: T) -> Self {
        Self { ident, name, value, dirty: true }
    }
}
