use qitech_lib::{machines::MachineIdentificationUnique, units::length};

use crate::{
    BoolProperty,
    FloatProperty,
    LengthProperty,
    PropertySet,
    PropertyEntry,
};

pub struct Allocator<'a> {
    ident: u64,
    set: &'a mut PropertySet,
}

impl<'a> Allocator<'a> {
    pub fn new(set: &'a mut PropertySet, ident: MachineIdentificationUnique) -> Self {
        Self { ident: ident.as_u64(), set }
    }

    pub fn add_bool(
        &mut self, 
        name: &'static str, 
        initial_value: bool
    ) -> Result<BoolProperty, AllocatorError> {
        let entry = PropertyEntry::new(self.ident, name, initial_value);
        let (value, dirty) = self.set.boolean.add(entry)?;
        Ok(BoolProperty::new(dirty, value))
    }

    pub fn create_float_property(
        &mut self, 
        name: &'static str, 
        initial_value: f64
    ) -> Result<FloatProperty, AllocatorError> {
        let entry = PropertyEntry::new(self.ident, name, initial_value);
        let (value, dirty) = self.set.float.add(entry)?;
        Ok(FloatProperty::new(dirty, value))
    }

    pub fn add_length<ExportUnit>(
        &mut self, 
        name: &'static str, 
        initial_value: f64,
    ) -> Result<LengthProperty<ExportUnit>, AllocatorError> 
    where 
        ExportUnit: length::Unit + uom::Conversion<f64, T = f64>
    {
        let entry = PropertyEntry::new(self.ident, name, initial_value);
        let (value, dirty) = self.set.float.add(entry)?;
        let inner = FloatProperty::new(dirty, value);
        Ok(LengthProperty::new(inner, initial_value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllocatorError;

impl core::fmt::Display for AllocatorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Pool is full")
    }
}

impl std::error::Error for AllocatorError {}