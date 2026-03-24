use std::borrow::Cow;

use crate::xtrem::protocol::DataAddress;

#[derive(Debug, Clone)]
pub struct Request<'a> {
    address: DataAddress,
    payload: RequestPayload<'a>
}

impl<'a> Request<'a> {
    pub fn read(address: DataAddress) -> Self {
        Request { 
            address, 
            payload: RequestPayload::Read,
        }
    }

    pub fn write(address: DataAddress, data: Cow<'a, [u8]>) -> Self {
        Request { 
            address, 
            payload: RequestPayload::Write(data),
        }
    }

    pub fn execute(address: DataAddress) -> Self {
        Request { 
            address, 
            payload: RequestPayload::Execute,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RequestPayload<'a> {
    Read,
    Write(Cow<'a, [u8]>),
    Execute,
}

#[derive(Debug, Clone)]
pub enum Response {
    Read(RegisterData),
    Write(u16),
    Execute(u16),
}

#[derive(Debug, Clone)]
pub enum RegisterData {
    SerialNumber(u32),
    DeviceId(u8),
    VccInputMinimum(),
    VccInputMaximum(),
    VccOutputMinimum(),
    VccOutputMaximum(),

    // 
    HardwareVersion(),
    SoftwareVersion(),
    SealingSwitchState(bool),

    //
    WeightValue(f64),
}

#[derive(Debug, Clone)]
pub enum DecodeError {
    InvalidUtf8,
    InvalidFormat,
    IntegerOverflow,
}

pub mod serial_number {
    use super::*;

    const ADDRESS: DataAddress = DataAddress::SerialNumber;

    pub fn read() -> Request<'static> { 
        Request::read(ADDRESS) 
    }
}

pub mod device_id {
    use super::*;

    const ADDRESS: DataAddress = DataAddress::DeviceId;

    pub fn read() -> Request<'static> { 
        Request::read(ADDRESS) 
    }
}

pub mod weight_value {
    use super::*;

    const ADDRESS: DataAddress = DataAddress::WeightValue;

    pub fn read() -> Request<'static> { 
        Request::read(ADDRESS) 
    }
}