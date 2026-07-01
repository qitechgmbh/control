use qitech_lib::{machines::MachineIdentificationUnique, units::length};
use crate::property::{FloatPool, IntPool};

use super::{
    BoolProperty,
    FloatProperty,
    LengthProperty,
    PropertyEntry,
};

pub struct PropertyAllocator<'a> {
    ident: u64,
    pool_f64: &'a mut FloatPool,
    pool_i64: &'a mut IntPool,
}

impl<'a> PropertyAllocator<'a> {
    pub fn new(
        pool_f64: &'a mut FloatPool,
        pool_i64: &'a mut IntPool,
        ident: MachineIdentificationUnique
    ) -> Self {
        Self { ident: ident.as_u64(), pool_f64, pool_i64 }
    }

    pub fn add_bool(
        &mut self, 
        name: &'static str, 
        initial_value: bool,
        always_dirty: bool,
    ) -> Result<BoolProperty, PropertyAllocatorError> {
        let initial_value = if initial_value {1} else {0};
        let entry = PropertyEntry::new(self.ident, name.into(), initial_value);
        let (value, dirty) = self.pool_i64.add(entry, always_dirty)?;
        Ok(BoolProperty::new(dirty, value))
    }

    pub fn create_float_property(
        &mut self, 
        name: &'static str, 
        initial_value: f64,
        always_dirty: bool,
    ) -> Result<FloatProperty, PropertyAllocatorError> {
        let entry = PropertyEntry::new(self.ident, name.into(), initial_value);
        let (value, dirty) = self.pool_f64.add(entry, always_dirty)?;
        Ok(FloatProperty::new(dirty, value))
    }

    pub fn add_length<ExportUnit>(
        &mut self, 
        name: &'static str, 
        initial_value: f64,
        always_dirty: bool,
    ) -> Result<LengthProperty<ExportUnit>, PropertyAllocatorError> 
    where 
        ExportUnit: length::Unit + uom::Conversion<f64, T = f64>
    {
        let entry = PropertyEntry::new(self.ident, name.into(), initial_value);
        let (value, dirty) = self.pool_f64.add(entry, always_dirty)?;
        let inner = FloatProperty::new(dirty, value);
        Ok(LengthProperty::new(inner, initial_value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertyAllocatorError;

impl core::fmt::Display for PropertyAllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Pool is full")
    }
}

impl std::error::Error for PropertyAllocatorError {}