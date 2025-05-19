pub mod coe;
pub mod pdo;

use anyhow::anyhow;
use coe::EL7031Configuration;
use ethercat_hal_derive::EthercatDevice;
use pdo::{EL7031RxPdo, EL7031TxPdo};

use crate::{
    io::{
        digital_input::{DigitalInputDevice, DigitalInputInput, DigitalInputState},
        stepper_velocity_el70x1::{
            StepperVelocityEL70x1Device, StepperVelocityEL70x1Input, StepperVelocityEL70x1Output,
            StepperVelocityEL70x1State,
        },
    },
    pdo::{PredefinedPdoAssignment, RxPdo, TxPdo},
    shared_config::el70x1::EL70x1OperationMode,
};

use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};

#[derive(Debug, EthercatDevice)]
pub struct EL7031 {
    pub txpdo: EL7031TxPdo,
    pub rxpdo: EL7031RxPdo,
    pub configuration: EL7031Configuration,
}

impl EthercatDeviceProcessing for EL7031 {}

impl NewEthercatDevice for EL7031 {
    fn new() -> Self {
        let configuration: EL7031Configuration = EL7031Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            configuration,
        }
    }
}

impl StepperVelocityEL70x1Device<EL7031StepperPort> for EL7031 {
    fn stepper_velocity_write(
        &mut self,
        port: EL7031StepperPort,
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
            EL7031StepperPort::STM1 => {
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
        port: EL7031StepperPort,
    ) -> Result<StepperVelocityEL70x1State, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7031StepperPort::STM1 => Ok(StepperVelocityEL70x1State {
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

impl DigitalInputDevice<EL7031DigitalInputPort> for EL7031 {
    fn digital_input_state(
        &self,
        port: EL7031DigitalInputPort,
    ) -> Result<DigitalInputState, anyhow::Error> {
        let error1 = anyhow::anyhow!(
            "[{}::Device::digital_input_state] Port {:?} is not available",
            module_path!(),
            port
        );
        Ok(DigitalInputState {
            input: DigitalInputInput {
                value: match port {
                    EL7031DigitalInputPort::DI1 => {
                        self.txpdo
                            .stm_status
                            .as_ref()
                            .ok_or(error1)?
                            .digital_input_1
                    }
                    EL7031DigitalInputPort::DI2 => {
                        self.txpdo
                            .stm_status
                            .as_ref()
                            .ok_or(error1)?
                            .digital_input_2
                    }
                },
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031StepperPort {
    STM1,
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031DigitalInputPort {
    DI1,
    DI2,
}

pub const EL7031_VENDOR_ID: u32 = 0x2;
pub const EL7031_PRODUCT_ID: u32 = 0x1b773052;
pub const EL7031_REVISION_A: u32 = 0x1A0000;
pub const EL7031_REVISION_B: u32 = 0x190000;
pub const EL7031_IDENTITY_A: SubDeviceIdentityTuple =
    (EL7031_VENDOR_ID, EL7031_PRODUCT_ID, EL7031_REVISION_A);
pub const EL7031_IDENTITY_B: SubDeviceIdentityTuple =
    (EL7031_VENDOR_ID, EL7031_PRODUCT_ID, EL7031_REVISION_B);
