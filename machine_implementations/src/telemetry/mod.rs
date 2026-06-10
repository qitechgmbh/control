use std::marker::PhantomData;

use anyhow::anyhow;
use qitech_lib::units::length::{self};

mod uom_properties;
pub use uom_properties::LengthProperty;

pub type BoolProperty = SimpleProperty<bool>;
pub type FloatProperty = SimpleProperty<f64>;
pub type IntProperty = SimpleProperty<i64>;

#[derive(Debug)]
pub struct SimpleProperty<T> {
    dirty: *mut bool,
    value: *mut T,
}

impl<T: Copy> SimpleProperty<T> {
    pub fn get(&self) -> T {
       unsafe { *self.value }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }

    pub fn is_dirty(&self) -> bool {
        unsafe { *self.dirty }
    }
}

#[derive(Debug)]
pub struct EnumProperty<T: Copy + From<i64> + Into<i64>> {
    dirty: *mut bool,
    value: *mut i64,
    phantom_data: PhantomData<T>,
}

impl<T: Copy + From<i64> + Into<i64>> EnumProperty<T> {
    pub fn get(&self) -> i64 {
       unsafe { *self.value }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            *self.value = value.into();
            *self.dirty = true;
        }
    }
}

#[derive(Debug)]
pub struct StringProperty {
    dirty: *mut bool,
    value: *mut heapless::String<128>,
}

impl StringProperty {
    pub fn get(&self) -> &heapless::String<128> {
       unsafe { &*self.value }
    }

    pub fn set(&mut self, value: heapless::String<128>) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }
}

#[derive(Debug, serde::Serialize)]
struct PropertySample<T> {
    ts: u64,
    vendor_id: u16,
    machine_id: u16,
    serial_id: u16,
    name: String,
    value: T,
}

#[derive(Debug, Default)]
pub struct PropertyPool {
    // must be heapless::Vec NOT Vec so the pointers remain valid!!
    properties_float: heapless::Vec<(&'static str, f64, bool), 128>,
    properties_int: heapless::Vec<(&'static str, i64, bool), 128>,
    properties_bool: heapless::Vec<(&'static str, bool, bool), 64>,
    properties_string: heapless::Vec<(&'static str, heapless::String<128>, bool), 32>,
}

impl PropertyPool {
    pub fn add_bool(
        &mut self, 
        name: &'static str, 
        initial_value: bool
    ) -> anyhow::Result<BoolProperty> {
        if self.properties_bool.push((name, initial_value, true)).is_err() {
            return Err(anyhow!("Float property slots full!"));
        }

        let item = self.properties_bool.last_mut().expect("Must contain item!");

        Ok(BoolProperty {
            dirty: &mut item.2 as *mut bool,
            value: &mut item.1 as *mut bool,
        })
    }

    pub fn create_float_property(
        &mut self, 
        name: &'static str, 
        initial_value: f64
    ) -> anyhow::Result<FloatProperty> {
        if self.properties_float.push((name, initial_value, true)).is_err() {
            return Err(anyhow!("Float property slots full!"));
        }

        let item = self.properties_float.last_mut().expect("Must contain item!");

        Ok(FloatProperty {
            dirty: &mut item.2 as *mut bool,
            value: &mut item.1 as *mut f64,
        })
    }

    pub fn add_length<ExportUnit: length::Unit + uom::Conversion<f64, T = f64>>(
        &mut self, 
        name: &'static str, 
        initial_value: f64,
    ) -> anyhow::Result<LengthProperty<ExportUnit>> {
        if self.properties_float.push((name, initial_value, true)).is_err() {
            return Err(anyhow!("Float property slots full!"));
        }

        let item = self.properties_float.last_mut().expect("Must contain item!");

        let inner = FloatProperty {
            dirty: &mut item.2 as *mut bool,
            value: &mut item.1 as *mut f64,
        };

        Ok(LengthProperty::new(inner, initial_value))
    }

    pub fn consume_dirty_float_properties<'a>(&'a mut self) -> FloatPropertyIterator<'a> {
        FloatPropertyIterator { 
            source: &mut self.properties_float, 
            index: 0 
        }
    }
}

pub struct FloatPropertyIterator<'a> {
    source: &'a mut heapless::Vec<(&'static str, f64, bool), 128>,
    index: usize,
}

impl<'a> Iterator for FloatPropertyIterator<'a> {
    type Item = (&'static str, f64);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(value) = self.source.get_mut(self.index) {
            self.index += 1;

            if value.2 {
                value.2 = false;
                return Some((value.0, value.1));
            }
        }

        None
    }
}
