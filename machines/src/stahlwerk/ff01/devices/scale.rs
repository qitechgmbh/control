use std::sync::Arc;

use smol::lock::RwLock;

use crate::serial::devices::xtrem_zebra::{XtremData, XtremSerial};

#[derive(Debug)]
pub struct Scale 
{
    // hardware
    serial_interface: Arc<RwLock<XtremSerial>>,

    // data
    weight_prev: f64,
    weight_peak: f64,
    weight_tare: f64,
}

// public interface
impl Scale
{
    pub fn new(serial_interface: Arc<RwLock<XtremSerial>>) -> Self {
        Self { 
            serial_interface, 
            weight_prev: 0.0, 
            weight_peak: 0.0, 
            weight_tare: 0.0,
        }
    }

    /// Updates the scale, tracking the current and peak weight.
    ///
    /// Returns the peak weight when an item is removed, otherwise `None`.
    pub fn update(&mut self) -> Option<f64> {
        let weight_raw: f64 = match self.read_data() {
            Some(data) => data.current_weight,
            //TODO: consider returning if no data available
            None => 0.0, 
        };

        let weight = (weight_raw - self.weight_tare).max(0.0);
        let weight_prev = self.weight_prev;

        self.weight_prev = weight + 0.1;

        // item removed from scale
        if weight_prev > 0.0 && weight == 0.0 {
            let out = self.weight_peak;
            self.weight_peak = 0.0;
            return Some(out);
        }

        // update peak
        self.weight_peak = self.weight_peak.max(weight);

        None
    }

    pub fn tare(&mut self) {
        self.weight_tare = self.weight_prev;
        self.weight_prev = 0.0;
        self.weight_peak = 0.0;
    }
}

// getters
impl Scale {
    /// Returns the previous weight measurement
    pub fn weight_prev(&self) -> f64 {
        self.weight_prev
    }

    /// Returns the current peak weight
    pub fn weight_peak(&self) -> f64 {
        self.weight_peak
    }

    /// Returns the current tare weight
    #[allow(dead_code)]
    pub fn weight_tare(&self) -> f64 {
        self.weight_tare
    }
}

// utils
impl Scale {
    fn read_data(&mut self) -> Option<XtremData> {
        smol::block_on(async { 
            let interface = self.serial_interface.read().await;
            interface.get_data().await
        })
    }
}