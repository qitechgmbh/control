use serde::{Serialize, Serializer, ser::SerializeStruct};

use crate::property::{FloatPool, IntPool, PropertyEntry};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub struct PropertyBatch {
    pub floats: Vec<PropertyEntry<f64>>,
    pub integers: Vec<PropertyEntry<i64>>,
}

pub struct PropertyBatchExporter<'a> {
    dirty_only: bool,
    pool_f64: &'a FloatPool,
    pool_i64: &'a IntPool,
}

impl<'a> PropertyBatchExporter<'a> {
    pub fn new(dirty_only: bool, pool_f64: &'a FloatPool, pool_i64: &'a IntPool) -> Self {
        Self { dirty_only, pool_f64, pool_i64 }
    }
}

impl Serialize for PropertyBatchExporter<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("PropertyBatch", 2)?;

        s.serialize_field(
            "floats",
            &self.pool_f64.iter(self.dirty_only).collect::<Vec<_>>(),
        )?;
        
        s.serialize_field(
            "integers",
            &self.pool_i64.iter(self.dirty_only).collect::<Vec<_>>(),
        )?;

        s.end()
    }
}
