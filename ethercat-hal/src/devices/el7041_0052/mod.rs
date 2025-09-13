use coe::EL7041_0052Configuration;
use ethercat_hal_derive::EthercatDevice;

use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};
use crate::{
    helpers::counter_wrapper_u16_i128::CounterWrapperU16U128,
    io::{
        digital_input::{DigitalInputDevice, DigitalInputInput},
        stepper_velocity_el70x1::{
            StepperVelocityEL70x1Device, StepperVelocityEL70x1Input, StepperVelocityEL70x1Output,
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
    is_used: bool,
    pub configuration: EL7041_0052Configuration,

    // encoder wrapping
    pub counter_wrapper: CounterWrapperU16U128,
}

impl NewEthercatDevice for EL7041_0052 {
    fn new() -> Self {
        let configuration = EL7041_0052Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
            configuration,
            counter_wrapper: CounterWrapperU16U128::new(),
        }
    }
}

impl EthercatDeviceProcessing for EL7041_0052 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        let enc_status_compact = match &self.txpdo.enc_status_compact {
            Some(value) => value,
            None => return Err(anyhow!("enc_status_compact is None")),
        };

        // update the counter wrapper
        self.counter_wrapper.update(
            enc_status_compact.counter_value,
            enc_status_compact.counter_underflow,
            enc_status_compact.counter_overflow,
        );

        Ok(())
    }

    fn output_pre_process(&mut self) -> Result<(), anyhow::Error> {
        let enc_status_compact = match &self.txpdo.enc_status_compact {
            Some(value) => value,
            None => return Err(anyhow!("enc_status_compact is None")),
        };

        let enc_control_compact = match &mut self.rxpdo.enc_control_compact {
            Some(value) => value,
            None => return Err(anyhow!("enc_control_compact is None")),
        };

        let stm_status = match &self.txpdo.stm_status {
            Some(value) => value,
            None => return Err(anyhow!("stm_status is None")),
        };

        let stm_control = match &mut self.rxpdo.stm_control {
            Some(value) => value,
            None => return Err(anyhow!("stm_control is None")),
        };

        // reset errors
        if stm_status.error {
            stm_control.reset = true;
        }

        // clear counter overflow/underflow flags by setting the counter to the current value
        if enc_status_compact.counter_overflow || enc_status_compact.counter_underflow {
            enc_control_compact.set_counter = true;
            enc_control_compact.set_counter_value = enc_status_compact.counter_value;
        }

        // set counter
        match self.counter_wrapper.pop_override() {
            Some(new_counter) => {
                enc_control_compact.set_counter = true;
                enc_control_compact.set_counter_value = new_counter;
            }
            None => {
                enc_control_compact.set_counter = false;
                enc_control_compact.set_counter_value = 0;
            }
        }

        Ok(())
    }
}

impl StepperVelocityEL70x1Device<EL7041_0052Port> for EL7041_0052 {
    fn set_output(
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
                // set the counter override if provided
                if let Some(new_counter) = value.set_counter {
                    self.counter_wrapper.push_override(new_counter);
                }

                match &mut self.rxpdo.stm_control {
                    Some(stm_control) => {
                        stm_control.enable = value.enable;
                        stm_control.reset = value.reset;
                        stm_control.reduce_torque = value.reduce_torque;
                    }
                    None => {
                        return Err(anyhow!("stm_control is None"));
                    }
                }
                match &mut self.rxpdo.stm_velocity {
                    Some(stm_velocity) => {
                        stm_velocity.velocity = value.velocity;
                    }
                    None => {
                        return Err(anyhow!("stm_velocity is None"));
                    }
                }
                Ok(())
            }
            _ => Err(anyhow!(
                "Port {:?} is not supported for stepper velocity",
                port
            )),
        }
    }

    fn get_input(
        &self,
        port: EL7041_0052Port,
    ) -> Result<StepperVelocityEL70x1Input, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7041_0052Port::STM1 => {
                let stm_status = match &self.txpdo.stm_status {
                    Some(value) => value,
                    None => return Err(anyhow!("stm_status is None")),
                };

                Ok(StepperVelocityEL70x1Input {
                    // Use the counter wrapper to get the current counter value
                    counter_value: self.counter_wrapper.current(),
                    ready_to_enable: stm_status.ready_to_enable,
                    ready: stm_status.ready,
                    warning: stm_status.warning,
                    error: stm_status.error,
                    moving_positive: stm_status.moving_positive,
                    moving_negative: stm_status.moving_negative,
                    torque_reduced: stm_status.torque_reduced,
                })
            }
            _ => Err(anyhow!(
                "Port {:?} is not supported for stepper velocity",
                port
            )),
        }
    }

    fn get_output(
        &self,
        port: EL7041_0052Port,
    ) -> Result<StepperVelocityEL70x1Output, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7041_0052Port::STM1 => {
                let stm_control = match &self.rxpdo.stm_control {
                    Some(value) => value,
                    None => return Err(anyhow!("stm_control is None")),
                };

                let stm_velocity = match &self.rxpdo.stm_velocity {
                    Some(value) => value,
                    None => return Err(anyhow!("stm_velocity is None")),
                };

                Ok(StepperVelocityEL70x1Output {
                    velocity: stm_velocity.velocity,
                    enable: stm_control.enable,
                    reduce_torque: stm_control.reduce_torque,
                    reset: stm_control.reset,
                    set_counter: self.counter_wrapper.get_override(),
                })
            }
            _ => Err(anyhow!(
                "Port {:?} is not supported for stepper velocity",
                port
            )),
        }
    }

    fn get_speed_range(
        &self,
        _port: EL7041_0052Port,
    ) -> crate::shared_config::el70x1::EL70x1SpeedRange {
        self.configuration.stm_features.speed_range
    }
}

impl DigitalInputDevice<EL7041_0052Port> for EL7041_0052 {
    fn get_input(&self, port: EL7041_0052Port) -> Result<DigitalInputInput, anyhow::Error> {
        let error1 = anyhow::anyhow!("stm_status is None");

        Ok(DigitalInputInput {
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
