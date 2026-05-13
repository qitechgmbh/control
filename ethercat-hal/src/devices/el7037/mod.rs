pub mod coe;
pub mod pdo;

use anyhow::anyhow;
use coe::EL7037Configuration;
use ethercat_hal_derive::EthercatDevice;
use pdo::{EL7037RxPdo, EL7037TxPdo};

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

use super::{EthercatDeviceProcessing, NewEthercatDevice, SubDeviceIdentityTuple};

#[derive(Debug, EthercatDevice)]
pub struct EL7037 {
    pub txpdo: EL7037TxPdo,
    pub rxpdo: EL7037RxPdo,
    is_used: bool,
    pub configuration: EL7037Configuration,
    pub counter_wrapper: CounterWrapperU16U128,
}

impl EthercatDeviceProcessing for EL7037 {
    fn input_post_process(&mut self) -> Result<(), anyhow::Error> {
        let enc_status_compact = match &self.txpdo.enc_status_compact {
            Some(value) => value,
            None => return Err(anyhow!("enc_status_compact is None")),
        };

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

impl NewEthercatDevice for EL7037 {
    fn new() -> Self {
        let configuration: EL7037Configuration = EL7037Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
            configuration,
            counter_wrapper: CounterWrapperU16U128::new(),
        }
    }
}

impl StepperVelocityEL70x1Device<EL7037StepperPort> for EL7037 {
    fn set_output(
        &mut self,
        port: EL7037StepperPort,
        value: StepperVelocityEL70x1Output,
    ) -> Result<(), anyhow::Error> {
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            panic!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            );
        }

        match port {
            EL7037StepperPort::STM1 => {
                if let Some(new_counter) = value.set_counter {
                    self.counter_wrapper.push_override(new_counter);
                }

                match &mut self.rxpdo.stm_control {
                    Some(stm_control) => {
                        stm_control.enable = value.enable;
                        stm_control.reduce_torque = value.reduce_torque;
                        stm_control.reset = value.reset;
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
        }
    }

    fn get_input(
        &self,
        port: EL7037StepperPort,
    ) -> Result<StepperVelocityEL70x1Input, anyhow::Error> {
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7037StepperPort::STM1 => {
                let stm_status = match &self.txpdo.stm_status {
                    Some(value) => value,
                    None => return Err(anyhow!("stm_status is None")),
                };

                Ok(StepperVelocityEL70x1Input {
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
        }
    }

    fn get_speed_range(
        &self,
        _port: EL7037StepperPort,
    ) -> crate::shared_config::el70x1::EL70x1SpeedRange {
        self.configuration.stm_features.speed_range
    }

    fn get_output(
        &self,
        port: EL7037StepperPort,
    ) -> Result<StepperVelocityEL70x1Output, anyhow::Error> {
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "Operation mode is not velocity, but {:?}",
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7037StepperPort::STM1 => {
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
        }
    }
}

impl DigitalInputDevice<EL7037DigitalInputPort> for EL7037 {
    fn get_input(&self, port: EL7037DigitalInputPort) -> Result<DigitalInputInput, anyhow::Error> {
        let error1 = anyhow::anyhow!(
            "[{}::Device::digital_input_state] Port {:?} is not available",
            module_path!(),
            port
        );
        Ok(DigitalInputInput {
            value: match port {
                EL7037DigitalInputPort::DI1 => {
                    self.txpdo
                        .stm_status
                        .as_ref()
                        .ok_or(error1)?
                        .digital_input_1
                }
                EL7037DigitalInputPort::DI2 => {
                    self.txpdo
                        .stm_status
                        .as_ref()
                        .ok_or(error1)?
                        .digital_input_2
                }
            },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL7037StepperPort {
    STM1,
}

#[derive(Debug, Clone, Copy)]
pub enum EL7037DigitalInputPort {
    DI1,
    DI2,
}

pub const EL7037_VENDOR_ID: u32 = 0x2;
pub const EL7037_PRODUCT_ID: u32 = 0x1b7d3052;
pub const EL7037_REVISION_A: u32 = 0x00170000;
pub const EL7037_IDENTITY_A: SubDeviceIdentityTuple =
    (EL7037_VENDOR_ID, EL7037_PRODUCT_ID, EL7037_REVISION_A);
