pub mod coe;
pub mod pdo;

use anyhow::anyhow;
use coe::EL7031_0030Configuration;
use ethercat_hal_derive::EthercatDevice;
use pdo::{EL7031_0030RxPdo, EL7031_0030TxPdo};
use units::{electric_potential::volt, f64::ElectricPotential};

use crate::{
    helpers::{
        counter_wrapper_u16_i128::CounterWrapperU16U128, signing_converter_u16::U16SigningConverter,
    },
    io::{
        analog_input::{AnalogInputDevice, AnalogInputInput, physical::AnalogInputRange},
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
pub struct EL7031_0030 {
    pub txpdo: EL7031_0030TxPdo,
    pub rxpdo: EL7031_0030RxPdo,
    is_used: bool,
    pub configuration: EL7031_0030Configuration,
    pub counter_wrapper: CounterWrapperU16U128,
}

impl EthercatDeviceProcessing for EL7031_0030 {
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

impl NewEthercatDevice for EL7031_0030 {
    fn new() -> Self {
        let configuration: EL7031_0030Configuration = EL7031_0030Configuration::default();
        Self {
            txpdo: configuration.pdo_assignment.txpdo_assignment(),
            rxpdo: configuration.pdo_assignment.rxpdo_assignment(),
            is_used: false,
            configuration,
            counter_wrapper: CounterWrapperU16U128::new(),
        }
    }
}

impl StepperVelocityEL70x1Device<EL7031_0030StepperPort> for EL7031_0030 {
    fn set_output(
        &mut self,
        port: EL7031_0030StepperPort,
        value: StepperVelocityEL70x1Output,
    ) -> Result<(), anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            panic!(
                "[{}::StepperVelocityEL70x1Device::stepper_velocity_write] Operation mode is not velocity, but {:?}",
                module_path!(),
                self.configuration.stm_features.operation_mode
            );
        }

        match port {
            EL7031_0030StepperPort::STM1 => {
                // set the counter override if provided
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
                        return Err(anyhow!(
                            "[{}::StepperVelocityEL70x1Device::stepper_velocity_write] stm_control is None",
                            module_path!()
                        ));
                    }
                }
                match &mut self.rxpdo.stm_velocity {
                    Some(stm_velocity) => {
                        stm_velocity.velocity = value.velocity;
                    }
                    None => {
                        return Err(anyhow!(
                            "[{}::StepperVelocityEL70x1Device::stepper_velocity_write] stm_velocity is None",
                            module_path!()
                        ));
                    }
                }
                Ok(())
            }
        }
    }

    fn get_input(
        &self,
        port: EL7031_0030StepperPort,
    ) -> Result<StepperVelocityEL70x1Input, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "[{}::StepperVelocityEL70x1Device::stepper_velocity_state] Operation mode is not velocity, but {:?}",
                module_path!(),
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7031_0030StepperPort::STM1 => {
                let stm_status = match &self.txpdo.stm_status {
                    Some(value) => value,
                    None => {
                        return Err(anyhow!(
                            "[{}::StepperVelocityEL70x1Device::stepper_velocity_state] stm_status is None",
                            module_path!()
                        ));
                    }
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

    fn get_output(
        &self,
        port: EL7031_0030StepperPort,
    ) -> Result<StepperVelocityEL70x1Output, anyhow::Error> {
        // check if operating mode is velocity
        if self.configuration.stm_features.operation_mode != EL70x1OperationMode::DirectVelocity {
            return Err(anyhow!(
                "[{}::StepperVelocityEL70x1Device::stepper_velocity_state] Operation mode is not velocity, but {:?}",
                module_path!(),
                self.configuration.stm_features.operation_mode
            ));
        }

        match port {
            EL7031_0030StepperPort::STM1 => {
                let stm_control = match &self.rxpdo.stm_control {
                    Some(value) => value,
                    None => {
                        return Err(anyhow!(
                            "[{}::StepperVelocityEL70x1Device::stepper_velocity_state] stm_control is None",
                            module_path!()
                        ));
                    }
                };

                let stm_velocity = match &self.rxpdo.stm_velocity {
                    Some(value) => value,
                    None => {
                        return Err(anyhow!(
                            "[{}::StepperVelocityEL70x1Device::stepper_velocity_state] stm_velocity is None",
                            module_path!()
                        ));
                    }
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

    fn get_speed_range(
        &self,
        _port: EL7031_0030StepperPort,
    ) -> crate::shared_config::el70x1::EL70x1SpeedRange {
        self.configuration.stm_features.speed_range
    }
}

impl DigitalInputDevice<EL7031_0030DigitalInputPort> for EL7031_0030 {
    fn get_input(
        &self,
        port: EL7031_0030DigitalInputPort,
    ) -> Result<DigitalInputInput, anyhow::Error> {
        let error1 = anyhow::anyhow!(
            "[{}::DigitalInputDevice::digital_input_state] StmStatus is None",
            module_path!(),
        );
        Ok(DigitalInputInput {
            value: match port {
                EL7031_0030DigitalInputPort::DI1 => {
                    self.txpdo
                        .stm_status
                        .as_ref()
                        .ok_or(error1)?
                        .digital_input_1
                }
                EL7031_0030DigitalInputPort::DI2 => {
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

impl AnalogInputDevice<EL7031_0030AnalogInputPort> for EL7031_0030 {
    fn get_input(&self, port: EL7031_0030AnalogInputPort) -> AnalogInputInput {
        let (raw_value, wiring_error) = match port {
            EL7031_0030AnalogInputPort::AI1 => match &self.txpdo {
                EL7031_0030TxPdo {
                    ai_standard_channel_1: Some(ai_standard_channel_1),
                    ..
                } => (ai_standard_channel_1.value, ai_standard_channel_1.error),
                EL7031_0030TxPdo {
                    ai_compact_channel_1: Some(ai_compact_channel_1),
                    ..
                } => (ai_compact_channel_1.value, false),
                _ => panic!("Invalid TxPdo assignment"),
            },
            EL7031_0030AnalogInputPort::AI2 => match &self.txpdo {
                EL7031_0030TxPdo {
                    ai_standard_channel_2: Some(ai_standard_channel_2),
                    ..
                } => (ai_standard_channel_2.value, ai_standard_channel_2.error),
                EL7031_0030TxPdo {
                    ai_compact_channel_2: Some(ai_compact_channel_2),
                    ..
                } => (ai_compact_channel_2.value, false),
                _ => panic!("Invalid TxPdo assignment"),
            },
        };
        let converted_raw_value = U16SigningConverter::load_raw(raw_value);
        let value: i16 = converted_raw_value.as_signed();

        let normalized = f32::from(value) / f32::from(i16::MAX);
        AnalogInputInput {
            normalized,
            wiring_error,
        }
    }

    fn analog_input_range(&self) -> AnalogInputRange {
        AnalogInputRange::Potential {
            min: ElectricPotential::new::<volt>(0.0),
            max: ElectricPotential::new::<volt>(10.0),
            min_raw: 0,
            max_raw: i16::MAX,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031_0030StepperPort {
    STM1,
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031_0030DigitalInputPort {
    DI1,
    DI2,
}

#[derive(Debug, Clone, Copy)]
pub enum EL7031_0030AnalogInputPort {
    AI1,
    AI2,
}

pub const EL7031_0030_VENDOR_ID: u32 = 0x2;
pub const EL7031_0030_PRODUCT_ID: u32 = 0x1b773052;
pub const EL7031_0030_REVISION_A: u32 = 0x10001E;
pub const EL7031_0030_IDENTITY_A: SubDeviceIdentityTuple = (
    EL7031_0030_VENDOR_ID,
    EL7031_0030_PRODUCT_ID,
    EL7031_0030_REVISION_A,
);
