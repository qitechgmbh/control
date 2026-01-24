use crate::serial::devices::us_3202510::register::HoldingRegister;
use crate::serial::devices::us_3202510::register::InputRegister;

use modbus::Request as ModbusRequest;

use proc_macros::{self, EnumCount};

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
    SetFrequency(u8),
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
    
    pub fn to_interface_request(&self) -> InterfaceRequest
    {
        match self
        {
            Request::RefreshStatus =>
            {
                InterfaceRequest {
                    type_id: self.tag() as usize,
                    payload: RequestPayload::ReadInputRegisters(ReadRegisters {
                        start_address: InputRegister::OFFSET,
                        quantity:      InputRegister::COUNT as u16,
                    }),
                }
            },
            Request::SetFrequency(value) =>
            {
                InterfaceRequest {
                    type_id: self.tag() as usize,
                    payload: RequestPayload::PresetHoldingRegister(WriteRegister {
                        address:  HoldingRegister::SetFrequency.address(),
                        value:    *value as u16,
                    }),
                }
            },
        }
    }
}

impl RequestType
{
    const fn registry_entry(&self) -> RequestRegistryEntry
    {
        match self
        {
            Self::RefreshStatus => RequestRegistryEntry { 
                priority:    0,
                extra_delay: 0,
            },
            Self::SetFrequency => RequestRegistryEntry { 
                priority:    1,
                extra_delay: 0,
            },
        }
    }
}

pub(crate) const REGISTRY: [RequestRegistryEntry; RequestType::COUNT] = [
    RequestType::RefreshStatus.registry_entry(),
    RequestType::SetFrequency.registry_entry(),
];