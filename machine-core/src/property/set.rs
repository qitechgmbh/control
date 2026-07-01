use super::PropertyPool;

use super::{
    POOL_CAPACITY_MAX_FLOAT,
    POOL_CAPACITY_MAX_INTEGER,
};

#[derive(Debug, Clone, Default)]
pub struct PropertySet {
    pub float: PropertyPool<f64, POOL_CAPACITY_MAX_FLOAT>,
    pub integer: PropertyPool<i64, POOL_CAPACITY_MAX_INTEGER>,
}

impl PropertySet {
    pub fn clear_dirty_flags(&mut self) {
        self.float.clear_dirty_flags();
        self.integer.clear_dirty_flags();
    }

    pub fn remove_where_machine_uid(&mut self, machine_uid: u64) {
        //TODO: implement
        _ = machine_uid;
    }
}
