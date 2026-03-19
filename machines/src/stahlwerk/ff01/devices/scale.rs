use std::sync::Arc;

use smol::lock::RwLock;

use crate::serial::devices::xtrem_zebra::{XtremData, XtremSerial};

#[derive(Debug)]
pub struct Scales 
{
    // hardware
    serial_interface: Arc<RwLock<XtremSerial>>,

    // data
    weight:      Option<f64>,
    weight_peak: Option<f64>,
    weight_tare: Option<f64>,
}

// public interface
impl Scales
{
    pub fn new(serial_interface: Arc<RwLock<XtremSerial>>) -> Self {
        Self { 
            serial_interface, 
            weight:      Some(0.0), 
            weight_peak: Some(0.0), 
            weight_tare: Some(0.0),
        }
    }

    /// Updates the scale, tracking the current and peak weight.
    ///
    /// Returns the peak weight when an item is removed, otherwise `None`.
    pub fn update(&mut self) {

        let weight_raw: f64 = match self.read_data() {
            Some(data) => data.current_weight,
            None => {
                self.weight = None;
                return;
            },
        };

        let weight_tare = self.weight_tare.unwrap_or(0.0);

        let weight = (weight_raw - weight_tare).max(0.0);

        self.weight = Some(weight);

        self.weight_peak = match self.weight_peak {
            Some(v) => Some(v.max(weight)),
            None => Some(weight),
        };
    }

    pub fn tare(&mut self) {
        let Some(weight_tare) = self.weight else {
            return;
        };

        self.weight_tare = Some(weight_tare);

        if self.weight.is_some() {
            self.weight = Some(0.0);
        }

        if let Some(weight_peak) = self.weight_peak {
            self.weight_peak = Some(weight_peak - weight_tare);
        }
    }

    pub fn tare_clear(&mut self) {
        self.weight = None;
        self.weight_peak = None;
        self.weight_tare = None;
    }

    pub fn weight_peak_reset(&mut self) {
        self.weight_peak = None;
    }   
}

// getters
impl Scales {
    pub fn weight(&self) -> Option<f64> {
        self.weight
    }

    pub fn weight_peak(&self) -> Option<f64> {
        self.weight_peak
    }

    #[allow(dead_code)]
    pub fn weight_tare(&self) -> Option<f64> {
        self.weight_tare
    }
}

// utils
impl Scales {
    fn read_data(&mut self) -> Option<XtremData> {
        smol::block_on(async { 
            let interface = self.serial_interface.read().await;
            interface.get_data().await
        })
    }
}