use coe::EL7041_0052Configuration;
use ethercat_hal_derive::EthercatDevice;

use super::{NewEthercatDevice, SubDeviceIdentityTuple};
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
use anyhow::anyhow;

pub mod coe;
pub mod pdo;

#[derive(EthercatDevice, Debug)]
pub struct EL7041_0052 {
    pub txpdo: pdo::EL7041_0052TxPdo,
    pub rxpdo: pdo::EL7041_0052RxPdo,
    pub configuration: EL7041_0052Configuration,
}

impl NewEthercatDevice for EL7041_0052 {
    fn new() -> Self {
        let configuration = EL7041_0052Configuration::default();
        EL7041_0052 {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            configuration,
        }
    }
}

impl StepperVelocityEL70x1Device<EL7041_0052Port> for EL7041_0052 {
    fn stepper_velocity_write(
        &mut self,
        port: EL7041_0052Port,
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
            EL7041_0052Port::STM1 => {
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
            _ => {
                return Err(anyhow!(
                    "Port {:?} is not supported for stepper velocity",
                    port
                ));
            }
        }
    }

    fn stepper_velocity_state(
        &self,
        port: EL7041_0052Port,
    ) -> Result<StepperVelocityEL70x1State, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7041_0052Port::STM1 => Ok(StepperVelocityEL70x1State {
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
            _ => {
                return Err(anyhow!(
                    "Port {:?} is not supported for stepper velocity",
                    port
                ));
            }
        }
    }
}

impl DigitalInputDevice<EL7041_0052Port> for EL7041_0052 {
    fn digital_input_state(
        &self,
        port: EL7041_0052Port,
    ) -> Result<DigitalInputState, anyhow::Error> {
        let error1 = anyhow::anyhow!(
            "[{}::Device::digital_input_state] Port {:?} is not available",
            module_path!(),
            port
        );
        Ok(DigitalInputState {
            input: DigitalInputInput {
                value: match port {
                    EL7041_0052Port::DI1 => {
                        self.txpdo
                            .stm_status
                            .as_ref()
                            .ok_or(error1)?
                            .digital_input_1
                    }
                    EL7041_0052Port::DI2 => {
                        self.txpdo
                            .stm_status
                            .as_ref()
                            .ok_or(error1)?
                            .digital_input_2
                    }
                    _ => {
                        return Err(anyhow!(
                            "Port {:?} is not supported for digital input",
                            port
                        ));
                    }
                },
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL7041_0052Port {
    STM1,
    DI1,
    DI2,
}

pub const EL7041_0052_VENDOR_ID: u32 = 0x2;
pub const EL7041_0052_PRODUCT_ID: u32 = 461451346;
pub const EL7041_0052_REVISION_A: u32 = 1048628;
pub const EL7041_0052_IDENTITY_A: SubDeviceIdentityTuple = (
    EL7041_0052_VENDOR_ID,
    EL7041_0052_PRODUCT_ID,
    EL7041_0052_REVISION_A,
);
