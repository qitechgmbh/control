use std::marker::PhantomData;
use qitech_lib::units::{Length, length};

use super::StringPropertyValue;

#[derive(Debug)]
pub struct SimpleProperty<T> {
    dirty: *mut bool,
    value: *mut T,
}

impl<T: Copy> SimpleProperty<T> {
    pub fn new(dirty: *mut bool, value: *mut T) -> Self {
       Self { dirty, value }
    }

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

pub type BoolProperty = SimpleProperty<bool>;
pub type FloatProperty = SimpleProperty<f64>;
pub type IntProperty = SimpleProperty<i64>;

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
    value: *mut StringPropertyValue,
}

impl StringProperty {
    pub fn get(&self) -> &StringPropertyValue {
       unsafe { &*self.value }
    }

    pub fn set(&mut self, value: StringPropertyValue) {
        unsafe {
            *self.value = value;
            *self.dirty = true;
        }
    }
}

// uom properties

#[derive(Debug)]
pub struct UomProperty<T: Copy + PartialOrd, ExportUnit> {
    inner: SimpleProperty<f64>,
    value: T,
    phantom_data: PhantomData<ExportUnit>,
}

// length
pub type LengthProperty<ExportUnit> = UomProperty<Length, ExportUnit>;

impl<ExportUnit: length::Unit + uom::Conversion<f64, T = f64>> LengthProperty<ExportUnit> {
    pub fn new(inner: SimpleProperty<f64>, initial_value: f64) -> Self {
        Self {
            inner,
            value: Length::new::<ExportUnit>(initial_value),
            phantom_data: PhantomData,
        }
    }

    pub fn get(&self) -> Length {
       self.value
    }

    pub fn get_as<U: length::Unit + uom::Conversion<f64, T = f64>>(&self) -> f64 {
       self.value.get::<U>()
    }

    pub fn set(&mut self, value: Length) {
        self.value = value;
        self.inner.set(self.get_as::<ExportUnit>());
    }

    pub fn set_as<U: length::Unit + uom::Conversion<f64, T = f64>>(&mut self, value: f64) {
        self.value = Length::new::<U>(value);
        self.inner.set(self.get_as::<ExportUnit>());
    }
}