use modbus::request::WriteRegister;
use strum::EnumCount;
use strum_macros::{EnumCount};

use crate::serial::devices::us_3202510::register::HoldingRegister;
use crate::serial::devices::us_3202510::register::InputRegister;

use modbus::Request as InterfaceRequest;

use modbus::request::ReadRegisters;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq, EnumCount)]
pub enum RequestType
{
    RefreshStatus,
    SetFrequency,
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Request
{
    RefreshStatus,
    SetFrequency(u16),
}

#[derive(Debug)]
pub struct RequestData
{
    pub priority: u32,
    pub payload:  InterfaceRequest,
}

impl Request
{
    pub fn tag(&self) -> RequestType
    {
        match self 
        {
            Request::RefreshStatus   => RequestType::RefreshStatus,
            Request::SetFrequency(_) => RequestType::SetFrequency,
        }
    }
    
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

                RequestData {
                    priority: 10,
                    payload,
                }
            }

            Request::SetFrequency(value) =>
            {
                let payload = 
                    InterfaceRequest::PresetHoldingRegister(
                        WriteRegister 
                        { 
                            address: HoldingRegister::SetFrequency as u16, 
                            value:   *value as u16,
                        }
                    );

                RequestData {
                    priority: 10,
                    payload,
                }
            }
        }
    }
}