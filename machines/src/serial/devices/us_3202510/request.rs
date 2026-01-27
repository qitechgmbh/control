use modbus::request::WriteRegister;
use strum::EnumCount;
use strum_macros::{EnumCount};

use crate::serial::devices::us_3202510::register::HoldingRegister;
use crate::serial::devices::us_3202510::register::InputRegister;

use modbus::Request as InterfaceRequest;

use modbus::request::ReadRegisters;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Request
{
    RefreshStatus,
    RefreshState,
    SetRotationState(u16),
    SetFrequencyTarget(u16),
    SetAccelerationLevel(u16),
    SetDecelerationLevel(u16),
}

#[derive(Debug)]
pub struct RequestData
{
    pub type_id:  u32,
    pub priority: u32,
    pub payload:  InterfaceRequest,
}

impl Request
{
    pub fn to_interface_request(&self) -> RequestData
    {
        match self
        {
            Request::RefreshStatus =>
            {
                let payload = 
                    InterfaceRequest::ReadInputRegisters(
                        ReadRegisters 
                        { 
                            start_address: InputRegister::OFFSET, 
                            quantity:      InputRegister::COUNT as u16,
                        }
                    );

                RequestData { type_id: 0, priority: 10, payload }
            }
            Request::RefreshState => 
            {
                let payload = 
                    InterfaceRequest::ReadHoldingRegisters(
                        ReadRegisters 
                        { 
                            start_address: HoldingRegister::OFFSET, 
                            quantity:      HoldingRegister::COUNT as u16,
                        }
                    );

                RequestData { type_id: 1, priority: 20, payload }
            }
            Request::SetRotationState(value) => 
            {
                let payload = 
                    InterfaceRequest::PresetHoldingRegister(
                        WriteRegister 
                        { 
                            address: HoldingRegister::RunCommand as u16, 
                            value: *value,
                        }
                    );

                RequestData { type_id: 2, priority: 100, payload }
            },
            Request::SetFrequencyTarget(value) =>
            {
                let payload = 
                    InterfaceRequest::PresetHoldingRegister(
                        WriteRegister 
                        { 
                            address: HoldingRegister::SetFrequency as u16, 
                            value:   *value,
                        }
                    );

                RequestData { type_id: 3, priority: 50, payload }
            }
            Request::SetAccelerationLevel(value) => 
            {
                let payload = 
                    InterfaceRequest::PresetHoldingRegister(
                        WriteRegister 
                        { 
                            address: HoldingRegister::AccelerationTime as u16, 
                            value:   *value,
                        }
                    );

                RequestData { type_id: 4, priority: 30, payload }
            },
            Request::SetDecelerationLevel(value) => 
            {
                let payload = 
                    InterfaceRequest::PresetHoldingRegister(
                        WriteRegister 
                        { 
                            address: HoldingRegister::DecelerationTime as u16, 
                            value:   *value,
                        }
                    );

                RequestData { type_id: 5, priority: 30, payload }
            },
        }
    }
}