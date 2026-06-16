use crate::{PropertyPool, StringPropertyValue};

use super::{
    POOL_CAPACITY_MAX_FLOAT,
    POOL_CAPACITY_MAX_INTEGER,
    POOL_CAPACITY_MAX_BOOLEAN,
    POOL_CAPACITY_MAX_STRING,
};

#[derive(Debug, Clone, Default)]
pub struct PropertySet {
    pub float:  PropertyPool<f64, POOL_CAPACITY_MAX_FLOAT>,
    pub int:    PropertyPool<i64, POOL_CAPACITY_MAX_INTEGER>,
    pub r#bool: PropertyPool<bool, POOL_CAPACITY_MAX_BOOLEAN>,
    pub string: PropertyPool<StringPropertyValue, POOL_CAPACITY_MAX_STRING>,
}

impl PropertySet {
    pub fn clear_dirty_flags(&mut self) {
        self.float.clear_dirty_flags();
        self.int.clear_dirty_flags();
        self.r#bool.clear_dirty_flags();
        self.string.clear_dirty_flags();
    }
}
