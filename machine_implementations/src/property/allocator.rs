use qitech_lib::{machines::MachineIdentificationUnique, units::length};

use super::{
    BoolProperty,
    FloatProperty,
    LengthProperty,
    Pool,
    PropertyEntry,
};

pub struct Allocator<'a> {
    pool: &'a mut Pool,
    ident: MachineIdentificationUnique,
}

impl<'a> Allocator<'a> {
    pub fn new(pool: &'a mut Pool, ident: MachineIdentificationUnique) -> Self {
        Self { pool, ident }
    }

    pub fn add_bool(
        &mut self, 
        name: &'static str, 
        initial_value: bool
    ) -> Result<BoolProperty, AllocatorError> {
        if self.pool.buf_bool.push(
            PropertyEntry::new(self.ident, name, initial_value)
        ).is_err() {
            return Err(AllocatorError);
        }

        let item = self.pool.buf_bool.last_mut().expect("Must contain item!");

        Ok(BoolProperty::new(
            &mut item.dirty as *mut bool, 
            &mut item.value as *mut bool
        ))
    }

    pub fn create_float_property(
        &mut self, 
        name: &'static str, 
        initial_value: f64
    ) -> Result<FloatProperty, AllocatorError> {
        if self.pool.buf_float.push(
            PropertyEntry::new(self.ident, name, initial_value)
        ).is_err() {
            return Err(AllocatorError);
        }

        let item = self.pool.buf_float.last_mut().expect("Must contain an item!");

        Ok(FloatProperty::new(
            &mut item.dirty as *mut bool, 
            &mut item.value as *mut f64
        ))
    }

    pub fn add_length<ExportUnit>(
        &mut self, 
        name: &'static str, 
        initial_value: f64,
    ) -> Result<LengthProperty<ExportUnit>, AllocatorError> 
    where 
        ExportUnit: length::Unit + uom::Conversion<f64, T = f64>
    {
        if self.pool.buf_float.push(
            PropertyEntry::new(self.ident, name, initial_value)
        ).is_err() {
            return Err(AllocatorError);
        }

        let item = self.pool.buf_float.last_mut().expect("Must contain item!");

        let inner = FloatProperty::new(
            &mut item.dirty as *mut bool, 
            &mut item.value as *mut f64
        );

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