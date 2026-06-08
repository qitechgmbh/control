use std::marker::PhantomData;

use qitech_lib::units::{Length, length::{centimeter, meter, millimeter}};

use crate::machine_identification::QiTechMachineIdentificationUnique;

#[derive(Debug)]
pub struct BoolProperty {
    dirty: *mut bool,
    value: *mut bool,
}

impl BoolProperty {
    pub fn get(&self) -> bool {
       unsafe { *self.value }
    }

    pub fn set(&mut self, value: bool) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }
}

pub trait EnumPropertyValue {
    fn as_u16(&self) -> u16;
}

#[derive(Debug)]
pub struct EnumProperty<T: Copy + From<u16> + Into<u16>> {
    dirty: *mut bool,
    value: *mut u16,
    phantom_data: PhantomData<T>,
}

impl<T: Copy + From<u16> + Into<u16>> EnumProperty<T> {
    pub fn get(&self) -> u16 {
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
    value: *mut String,
}

impl StringProperty {
    pub fn get(&self) -> &String {
       unsafe { &*self.value }
    }

    pub fn set(&mut self, value: String) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }
}

#[derive(Debug)]
pub struct UomProperty<T: Copy + PartialOrd> {
    dirty: *mut bool,
    data: *mut NumericPropertyData<f64>,
    value: T,
    convert: fn(T) -> f64,
}

impl<T: Copy + PartialOrd> UomProperty<T> {
    pub fn get<U>(&self) -> T {
       self.value
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            let val_f64 = (self.convert)(value);
            let data = &mut *self.data;

            data.now = val_f64;

            if val_f64 < data.min {
                data.min = val_f64;
            } else if val_f64 > data.max {
                data.max = val_f64;
            }
            
            *self.dirty = true;
        }
    }
}

#[derive(Debug)]
pub struct NumericProperty<T: Copy + PartialOrd> {
    dirty: *mut bool,
    data: *mut NumericPropertyData<T>,
}

impl<T: Copy + PartialOrd> NumericProperty<T> {
    pub fn get(&self) -> T {
       unsafe { (*self.data).now }
    }

    pub fn set(&mut self, value: T) {
        unsafe {
            let data = &mut *self.data;

            data.now = value;

            if value < data.min {
                data.min = value;
            } else if value > data.max {
                data.max = value;
            }
            
            *self.dirty = true;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NumericPropertyData<T: Copy + PartialOrd> {
    pub now: T,
    pub min: T,
    pub max: T,
}

#[derive(Debug)] 
pub struct PropertyInfo {
    pub name: String, 
    pub desc: String,
}

pub type FloatProperty = NumericProperty<f64>;
pub type PropertyDataF64 = NumericPropertyData<f64>;

pub type PropertyI64 = NumericProperty<i64>;
pub type PropertyDataI64 = NumericPropertyData<i64>;

#[derive(Debug)]
pub struct PropertyManager {
    machine_uid: QiTechMachineIdentificationUnique,

    // must be heapless so the pointers remain valid!!
    properties_float: heapless::Vec<(String, PropertyDataF64, bool), 128>,

    // properties_integer: Vec<i64>,
    properties_bool: heapless::Vec<(String, bool, bool), 128>,
    // properties_enum:    Vec<u16>,
    // properties_string:  Vec<String>,
}

impl PropertyManager {
    pub fn create_bool_property(
        &mut self, 
        name: &'static str, 
        initial_value: bool
    ) -> Option<BoolProperty> {
        if self.properties_bool.push((name.into(), initial_value, true)).is_err() {
            return None;
        }

        let item = self.properties_bool.last_mut().expect("Must contain item!");

        Some(BoolProperty {
            dirty: &mut item.2 as *mut bool,
            value: &mut item.1 as *mut bool,
        })
    }

    pub fn create_float_property(
        &mut self, 
        name: &'static str, 
        initial_value: f64
    ) -> Option<FloatProperty> {
        let data = PropertyDataF64 { 
            now: initial_value, 
            min: initial_value,
            max: initial_value,
        };

        if self.properties_float.push((name.into(), data, true)).is_err() {
            return None;
        }

        let item = self.properties_float.last_mut().expect("Must contain item!");

        Some(FloatProperty {
            dirty: &mut item.2 as *mut bool,
            data: &mut item.1 as *mut PropertyDataF64,
        })
    }

    pub fn create_length_property(
        &mut self, 
        name: &'static str, 
        initial_value: Length,
        export_unit: LengthUnit,
    ) -> Option<LengthProperty> {
        let convert = match export_unit {
            LengthUnit::Millimeter => millimeter_to_f64,
            LengthUnit::Centimeter => centimeter_to_f64,
            LengthUnit::Meter => meter_to_f64,
        };

        let value_f64 = (convert)(initial_value);

        let data = PropertyDataF64 { 
            now: value_f64, 
            min: value_f64,
            max: value_f64,
        };

        if self.properties_float.push((name.into(), data, true)).is_err() {
            return None;
        }

        let item = self.properties_float.last_mut().expect("Must contain item!");

        Some(LengthProperty {
            dirty: &mut item.2 as *mut bool,
            data: &mut item.1 as *mut PropertyDataF64,
            value: initial_value,
            convert,
        })
    }

    pub fn extract_float_properties<'a>(&'a mut self) -> FloatPropertyIterator<'a> {
        FloatPropertyIterator { 
            source: &mut self.properties_float, 
            index: 0 
        }
    }
}

pub fn millimeter_to_f64(value: Length) -> f64 {
    return value.get::<millimeter>()
}

pub fn centimeter_to_f64(value: Length) -> f64 {
    return value.get::<centimeter>()
}

pub fn meter_to_f64(value: Length) -> f64 {
    return value.get::<meter>()
}

pub struct FloatPropertyIterator<'a> {
    source: &'a mut heapless::Vec<(String, PropertyDataF64, bool), 128>,
    index: usize,
}

impl<'a> Iterator for FloatPropertyIterator<'a> {
    type Item = (String, PropertyDataF64);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(value) = self.source.get_mut(self.index) {
            self.index += 1;

            if value.2 {
                value.2 = false;
                return Some((value.0.clone(), value.1));
            }
        }

        None
    }
}

// LengthProperty: create_length_property(millimeter)

#[derive(Debug, Clone)]
pub enum LengthUnit {
    Millimeter,
    Centimeter,
    Meter,
}

pub type LengthProperty = UomProperty<Length>;