
use modbus::{RequestResult, rtu::{DispatchError, backends::LinuxTransport}};
// external deps
use serde::{Deserialize, Serialize};

// use modbus::ExceptionCode

type ModbusInterface = modbus::rtu::Interface<CustomTransport, (), 25>;

// internal deps
pub use request::Request;

use units::{Frequency, electric_current::ampere, electric_potential::volt, frequency::hertz, thermodynamic_temperature::degree_celsius};

// modules
mod register;
mod request;
mod serial_device;

mod transport;

use crate::serial::devices::us_3202510::{register::InputRegister, transport::CustomTransport};

#[derive(Debug)]
pub struct US3202510
{
    pub path: String,
    pub config: Config,
    pub status: Option<Status>,
    
    failed_attempts: u8,
    interface: ModbusInterface,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Config 
{
    pub rotation_state: RotationState,
    pub frequency: units::Frequency, // 0 - 99hz
    pub acceleration_level: u16, // 1 - 15
    pub deceleration_level: u16, // 1 - 15
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Status
{
    pub frequency: units::Frequency, // 1 - 99hz
    pub voltage: units::ElectricPotential,
    pub current: units::ElectricCurrent,
    pub temperature: units::ThermodynamicTemperature,
    pub operation_status: OperationStatus,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationState
{
    #[default]
    Stopped,
    Forward,
    Reverse,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OperationStatus
{
    #[default]
    Idle,
    Running,
    Fault,
}

impl US3202510 
{
    pub fn update(&mut self)
    {
        match self.interface.await_result()
        {
            Ok(result) => 
            {
                tracing::error!("has result: {:?}", &result);

                self.handle_result(result);
            },
            Err(e) => 
            {
                match e 
                {
                    modbus::rtu::ReceiveError::NoPendingRequest => {},

                    e => 
                    {
                        //tracing::error!("Error reciving result: {:?}", e);
                    }
                }
            },
        }
        
        //TODO: anyhow crashes system for reason?
        // self.refresh_status();
        
        if !self.interface.has_pending_request()
        {
            match self.interface.dispatch_next_request()
            {
                Ok(_) => 
                {  
                    // tracing::error!("Success sending request: {:?}", self.interface);
                },
                Err(e) => 
                { 
                    match e
                    {
                        DispatchError::RequestPending => {},
                        DispatchError::QueueEmpty => {},
                        DispatchError::Transport(err) => 
                        {
                            tracing::error!("Error sending request: {}", err);
                        },
                        DispatchError::BridgeDropped => 
                        {
                            tracing::error!("Bridge dropped");
                        },
                    }
                },
            }
        }
    }
    
    fn queue_request(&mut self, request: Request)
    {
        let data = request.to_interface_request();
        match self.interface.queue_request((), data.payload, data.priority)
        {
            Ok(_) => {},
            Err(x) => 
            {
                match x
                {
                    modbus::QueueItemError::QueueFull     => { tracing::error!("Failed to put item into queue!"); },
                    modbus::QueueItemError::DuplicateItem => { tracing::error!("Failed to put item into queue!"); },
                }
            },
        }
    }
    
    pub fn refresh_status(&mut self)
    {
        self.queue_request(Request::RefreshStatus);
    }

    pub fn set_frequency_target(&mut self, frequency: units::Frequency)
    {

        tracing::error!("Set frequency");

        self.config.frequency = frequency;
        let as_hertz_u8 = frequency.get::<hertz>().round().clamp(0.0, 99.0) as u8;
        self.queue_request(Request::SetFrequency(as_hertz_u8));
    }

/*
    pub fn set_rotation_state(&mut self, rotation_state: RotationState)
    {
        self.config.rotation_state = rotation_state;
        self.queue_request(Request::StartForwardRotation);
    }
    
    pub fn set_frequency_target(&mut self, frequency: units::Frequency)
    {
        self.config.frequency = frequency;
        self.queue_request(Request::StartForwardRotation);
    }
    
    pub fn set_acceleration_level(&mut self, acceleration_level: u16)
    {
        self.config.acceleration_level = acceleration_level;
        self.queue_request(Request::StartForwardRotation);
    }
    
    pub fn set_deceleration_level(&mut self, deceleration_level: u16)
    {
        self.config.deceleration_level = deceleration_level;
        self.queue_request(Request::StartForwardRotation);
    }
    */
    
    fn handle_result(&mut self, response: RequestResult)
    {
        match response 
        {
            RequestResult::ReadHoldingRegisters(_) => 
            {
                tracing::error!("ReadHoldingRegisters: WORKED");
            },
            RequestResult::ReadInputRegisters(data) => 
            {
                let frequency: u16 = data.result.get(InputRegister::CurrentFrequency as usize).unwrap_or(&0) / 10;
                
                let voltage: u16 = data.result.get(InputRegister::BusVoltage as usize).unwrap_or(&0) / 10;
                
                let current: u16 = data.result.get(InputRegister::LineCurrent as usize).unwrap_or(&0) / 100;
                
                let temperature: u16 = *data.result.get(InputRegister::DriveTemperature as usize).unwrap_or(&0) / 10000;
                
                let operation_status: u16 = *data.result.get(InputRegister::SystemStatus as usize).unwrap_or(&0);
                
                
                _ = operation_status;
                
                self.status = Some(Status {
                    frequency:        units::Frequency::new::<hertz>(0 as f64),
                    voltage:          units::ElectricPotential::new::<volt>(0 as f64),
                    current:          units::ElectricCurrent::new::<ampere>(0 as f64),
                    temperature:      units::ThermodynamicTemperature::new::<degree_celsius>(0 as f64),
                    operation_status: OperationStatus::Idle,
                });
            },
            RequestResult::PresetHoldingRegister(request_result_data) => 
            {
                tracing::error!("PresetHoldingRegister: WORKED");
            },
            RequestResult::Exception(request_result_data) => 
            {
                tracing::error!("Received exception: {:?}", request_result_data.result);
            },
        }   
    }
}

#[cfg(test)]
mod tests 
{
    #[test]
    fn test_requests() 
    {
        assert!(1 == 2);
    }
}