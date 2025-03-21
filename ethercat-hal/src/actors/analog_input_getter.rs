use super::Actor;
use crate::io::analog_input::AnalogInput;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::RwLock;
use uom::si::{
    electric_current::milliampere,
    electric_potential::volt,
    f32::{ElectricCurrent, ElectricPotential},
};

#[derive(Debug)]
pub enum AnalogInputValue {
    Potential(ElectricPotential),
    Current(ElectricCurrent),
}

#[derive(Debug)]
pub enum AnalogInputRange {
    Potential {
        min: ElectricPotential,
        max: ElectricPotential,
    },
    Current {
        min: ElectricCurrent,
        max: ElectricCurrent,
    },
}

pub enum AnalogInputDevice {
    EL300x,
    EL304x,
    EL305x,
    EL306x,
    EL3062_0030,
}

impl From<AnalogInputDevice> for AnalogInputRange {
    fn from(device: AnalogInputDevice) -> Self {
        match device {
            AnalogInputDevice::EL300x => AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(-10.0),
                max: ElectricPotential::new::<volt>(10.0),
            },
            AnalogInputDevice::EL304x => AnalogInputRange::Current {
                min: ElectricCurrent::new::<milliampere>(0.0),
                max: ElectricCurrent::new::<milliampere>(20.0),
            },
            AnalogInputDevice::EL305x => AnalogInputRange::Current {
                min: ElectricCurrent::new::<milliampere>(4.0),
                max: ElectricCurrent::new::<milliampere>(20.0),
            },
            AnalogInputDevice::EL306x => AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(0.0),
                max: ElectricPotential::new::<volt>(10.0),
            },
            AnalogInputDevice::EL3062_0030 => AnalogInputRange::Potential {
                min: ElectricPotential::new::<volt>(0.0),
                max: ElectricPotential::new::<volt>(30.0),
            },
        }
    }
}

/// Log the state of a analog input
#[derive(Debug)]
pub struct AnalogInputGetter {
    input: AnalogInput,
    normalized: Option<f32>,
    pub range: AnalogInputRange,
}

impl AnalogInputGetter {
    pub fn new(input: AnalogInput, range: AnalogInputRange) -> Self {
        Self {
            input,
            normalized: None,
            range,
        }
    }

    pub fn get_normalized(&self) -> Option<f32> {
        self.normalized
    }

    pub fn get_physical(&self) -> Option<AnalogInputValue> {
        match self.range {
            AnalogInputRange::Potential { min, max } => {
                let normalized = self.get_normalized()?;
                let value = min + (max - min) * normalized;
                Some(AnalogInputValue::Potential(value))
            }
            AnalogInputRange::Current { min, max } => {
                let normalized = self.get_normalized()?;
                let value = min + (max - min) * normalized;
                Some(AnalogInputValue::Current(value))
            }
        }
    }
}

impl Actor for AnalogInputGetter {
    fn act(&mut self, _now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let state = (self.input.state)().await;
            self.normalized = Some(state.input.normalized);
        })
    }
}

impl From<AnalogInputGetter> for Arc<RwLock<AnalogInputGetter>> {
    fn from(actor: AnalogInputGetter) -> Self {
        Arc::new(RwLock::new(actor))
    }
}
