pub mod coe;
pub mod pdo;

use anyhow::anyhow;
use coe::EL7031_0030Configuration;
use ethercat_hal_derive::EthercatDevice;
use pdo::{EL7031_0030RxPdo, EL7031_0030TxPdo};
use uom::si::{electric_potential::volt, f64::ElectricPotential};

use crate::{
    io::{
        analog_input::{
            AnalogInputDevice, AnalogInputInput, AnalogInputState, physical::AnalogInputRange,
        },
        stepper_velocity_el70x1::{
            StepperVelocityEL70x1Device, StepperVelocityEL70x1Input, StepperVelocityEL70x1Output,
            StepperVelocityEL70x1State,
        },
    },
    pdo::{PredefinedPdoAssignment, RxPdo, TxPdo},
    shared_config::{el30xx::EL30XXPresentation, el70x1::EL70x1OperationMode},
    signing::U16SigningConverter,
};

use super::{NewEthercatDevice, SubDeviceIdentityTuple};

#[derive(Debug, EthercatDevice)]
pub struct EL7031_0030 {
    pub txpdo: EL7031_0030TxPdo,
    pub rxpdo: EL7031_0030RxPdo,
    pub configuration: EL7031_0030Configuration,
}

impl NewEthercatDevice for EL7031_0030 {
    fn new() -> Self {
        let configuration: EL7031_0030Configuration = EL7031_0030Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            configuration,
        }
    }
}

impl StepperVelocityEL70x1Device<EL7031_0030StepperPort> for EL7031_0030 {
    fn stepper_velocity_write(
        &mut self,
        port: EL7031_0030StepperPort,
        value: StepperVelocityEL70x1Output,
    ) -> Result<(), anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            panic!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            );
        }

        match port {
            EL7031_0030StepperPort::STM1 => {
                match &mut self.rxpdo.enc_control_compact {
                    Some(before) => *before = value.enc_control_compact,
                    None => {
                        return Err(anyhow!("enc_status_compact is None"));
                    }
                }
                match &mut self.rxpdo.stm_control {
                    Some(before) => *before = value.stm_control,
                    None => {
                        return Err(anyhow!("stm_control is None"));
                    }
                }
                match &mut self.rxpdo.stm_velocity {
                    Some(before) => *before = value.stm_velocity,
                    None => {
                        return Err(anyhow!("stm_velocity is None"));
                    }
                }
                Ok(())
            }
        }
    }

    fn stepper_velocity_state(
        &self,
        port: EL7031_0030StepperPort,
    ) -> Result<StepperVelocityEL70x1State, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7031_0030StepperPort::STM1 => Ok(StepperVelocityEL70x1State {
                input: StepperVelocityEL70x1Input {
                    enc_status_compact: match &self.txpdo.enc_status_compact {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("enc_status_compact is None")),
                    },
                    stm_status: match &self.txpdo.stm_status {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("stm_status is None")),
                    },
                },
                output: StepperVelocityEL70x1Output {
                    enc_control_compact: match &self.rxpdo.enc_control_compact {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("enc_control_compact is None")),
                    },
                    stm_control: match &self.rxpdo.stm_control {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("stm_control is None")),
                    },
                    stm_velocity: match &self.rxpdo.stm_velocity {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("stm_velocity is None")),
                    },
                },
            }),
        }
    }
}

impl AnalogInputDevice<EL7031_0030AnalogInputPort> for EL7031_0030 {
    fn analog_output_state(&self, port: EL7031_0030AnalogInputPort) -> AnalogInputState {
        let raw_value = match port {
            EL7031_0030AnalogInputPort::AI1 => match &self.txpdo {
                EL7031_0030TxPdo {
                    ai_standard_channel_1: Some(ai_standard_channel_1),
                    ..
                } => ai_standard_channel_1.value,
                EL7031_0030TxPdo {
                    ai_compact_channel_1: Some(ai_compact_channel_1),
                    ..
                } => ai_compact_channel_1.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
            EL7031_0030AnalogInputPort::AI2 => match &self.txpdo {
                EL7031_0030TxPdo {
                    ai_standard_channel_2: Some(ai_standard_channel_2),
                    ..
                } => ai_standard_channel_2.value,
                EL7031_0030TxPdo {
                    ai_compact_channel_2: Some(ai_compact_channel_2),
                    ..
                } => ai_compact_channel_2.value,
                _ => panic!("Invalid TxPdo assignment"),
            },
        };
        let raw_value = U16SigningConverter::load_raw(raw_value);
        println!("{}", raw_value);

        let presentation = match port {
            EL7031_0030AnalogInputPort::AI1 => {
                self.configuration.analog_input_channel_1.presentation
            }
            EL7031_0030AnalogInputPort::AI2 => {
                self.configuration.analog_input_channel_2.presentation
            }
        };

        let value: i16 = match presentation {
            EL30XXPresentation::Unsigned => raw_value.as_unsigned() as i16,
            EL30XXPresentation::Signed => raw_value.as_signed(),
            EL30XXPresentation::SignedMagnitude => raw_value.as_signed_magnitude(),
        };

        let normalized = f32::from(value) / f32::from(i16::MAX);
        AnalogInputState {
            input: AnalogInputInput { normalized },
        }
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031_0030StepperPort {
    STM1,
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031_0030AnalogInputPort {
    AI1,
    AI2,
}

pub const EL7031_0030_VENDOR_ID: u32 = 0x2;
pub const EL7031_0030_PRODUCT_ID: u32 = 0x1b773052;
pub const EL7031_0030_REVISION_A: u32 = 0x1A0000;
pub const EL7031_0030_REVISION_B: u32 = 0x190000;
pub const EL7031_0030_IDENTITY_A: SubDeviceIdentityTuple = (
    EL7031_0030_VENDOR_ID,
    EL7031_0030_PRODUCT_ID,
    EL7031_0030_REVISION_A,
);
pub const EL7031_0030_IDENTITY_B: SubDeviceIdentityTuple = (
    EL7031_0030_VENDOR_ID,
    EL7031_0030_PRODUCT_ID,
    EL7031_0030_REVISION_B,
);
