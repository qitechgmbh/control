use std::marker::PhantomData;

use qitech_lib::units::{Length, length::{self}};

use crate::telemetry::SimpleProperty;

#[derive(Debug)]
pub struct UomProperty<T: Copy + PartialOrd, ExportUnit> {
    inner: SimpleProperty<f64>,
    value: T,
    phantom_data: PhantomData<ExportUnit>,
}

pub type LengthProperty<ExportUnit: length::Unit + uom::Conversion<f64, T = f64>> = UomProperty<Length, ExportUnit>;

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

    pub fn set<U: length::Unit + uom::Conversion<f64, T = f64>>(&mut self, value: f64) {
        self.value = Length::new::<U>(value);
        self.inner.set(self.get_as::<ExportUnit>());
    }
}

