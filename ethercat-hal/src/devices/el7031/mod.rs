pub mod coe;
pub mod pdo;

use anyhow::anyhow;
use coe::{EL7031Configuration, EL7031OperationMode};
use ethercat_hal_derive::Device;
use pdo::{EL7031RxPdo, EL7031TxPdo};

use crate::{
    io::stepper_velocity_el7031::{
        StepperVelocityEL7031Device, StepperVelocityEL7031Input, StepperVelocityEL7031Output,
        StepperVelocityEL7031State,
    },
    pdo::{PredefinedPdoAssignment, RxPdo, TxPdo},
};

use super::{NewDevice, SubDeviceIdentityTuple};

#[derive(Debug, Device)]
pub struct EL7031 {
    pub txpdo: EL7031TxPdo,
    pub output_ts: u64,
    pub input_ts: u64,
    pub rxpdo: EL7031RxPdo,
    pub configuration: EL7031Configuration,
}

impl NewDevice for EL7031 {
    fn new() -> Self {
        let configuration: EL7031Configuration = EL7031Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            input_ts: 0,
            output_ts: 0,
            configuration,
        }
    }
}

impl StepperVelocityEL7031Device<EL7031Port> for EL7031 {
    fn stepper_velocity_write(
        &mut self,
        port: EL7031Port,
        value: StepperVelocityEL7031Output,
    ) -> Result<(), anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL7031OperationMode::DirectVelocity {
            panic!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            );
        }

        match port {
            EL7031Port::STM1 => {
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
        port: EL7031Port,
    ) -> Result<StepperVelocityEL7031State, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL7031OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7031Port::STM1 => Ok(StepperVelocityEL7031State {
                output_ts: self.output_ts,
                input_ts: self.input_ts,
                input: StepperVelocityEL7031Input {
                    enc_status_compact: match &self.txpdo.enc_status_compact {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("enc_status_compact is None")),
                    },
                    stm_status: match &self.txpdo.stm_status {
                        Some(value) => value.clone(),
                        None => return Err(anyhow!("stm_status is None")),
                    },
                },
                output: StepperVelocityEL7031Output {
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

#[derive(Debug, Clone, Copy)]
pub enum EL7031Port {
    STM1,
}

pub const EL7031_VENDOR_ID: u32 = 0x2;
pub const EL7031_PRODUCT_ID: u32 = 460795986;
pub const EL7031_REVISION_A: u32 = 1703936;
pub const EL7031_IDENTITY_A: SubDeviceIdentityTuple =
    (EL7031_VENDOR_ID, EL7031_PRODUCT_ID, EL7031_REVISION_A);
