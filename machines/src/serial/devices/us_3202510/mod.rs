
use modbus::{RequestResult, rtu::{DispatchError}};
// external deps
use serde::{Deserialize, Serialize};

// use modbus::ExceptionCode

// internal deps
pub use request::Request;

use units::{Frequency, electric_current::ampere, electric_potential::volt, frequency::hertz, thermodynamic_temperature::{degree_celsius, kelvin}};

// modules
mod register;
mod request;
mod serial_device;

mod transport;

use crate::serial::devices::us_3202510::{register::{HoldingRegister, InputRegister}, transport::{CustomTransport, EntryContext}};

type ModbusInterface = modbus::rtu::Interface<CustomTransport, EntryContext, 25>;

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
    pub snapshot_id: u64,
    
    pub running: bool,
    pub direction: bool, 
    pub frequency_target: u16, // 0 - 99hz
    pub frequency_min: u16,
    pub frequency_max: u16,
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
pub enum Direction
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
        if self.config.snapshot_id == 0
        {
            self.refresh_config();
            self.config.snapshot_id += 1;
        }
        
        match self.interface.await_result()
        {
            Ok(result) => 
            {
                self.handle_result(result);
            },
            Err(e) => 
            {
                match e 
                {
                    modbus::rtu::ReceiveError::NoPendingRequest => {},

                    e => 
                    {
                        tracing::error!("Error reciving result: {:?}", e);
                    }
                }
            },
        }
        
        if !self.interface.has_pending_request()
        {
            match self.interface.dispatch_next_request()
            {
                Ok(_) => {},
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
        match self.interface.queue_request(data.type_id, data.payload, data.priority)
        {
            Ok(_) => {},
            Err(x) => 
            {
                match x
                {
                    modbus::QueueItemError::QueueFull     => { tracing::error!("QueueFull!"); },
                    modbus::QueueItemError::DuplicateItem => { tracing::error!("DuplicateItem!"); },
                }
            },
        }
    }
    
    pub fn refresh_status(&mut self)
    {
        self.queue_request(Request::RefreshStatus);
    }

    pub fn refresh_config(&mut self)
    {
        self.queue_request(Request::RefreshState);
    }

    pub fn set_running(&mut self, running: bool)
    {
        if self.config.running == running { return; }
        
        match running 
        {
            true => 
            { 
                let value: u16 = match self.config.direction 
                {
                    true  => 1,
                    false => 3,
                };
                
                self.queue_request(Request::SetRotationState(value)); 
            },
            false => { self.queue_request(Request::SetRotationState(2)); },
        }

        self.config.running = running;
    }

    pub fn set_direction(&mut self, direction: bool)
    {
        if self.config.direction != direction && self.config.running
        {
            let value: u16 = if direction { 1 } else { 3 };
            
            self.queue_request(Request::SetRotationState(value));
        }
        
        self.config.direction = direction;
    }

    pub fn set_frequency_target(&mut self, frequency: u16)
    {
        self.config.frequency_target = frequency.min(990);
        self.queue_request(Request::SetFrequencyTarget(self.config.frequency_target));
    }
    
    pub fn set_acceleration_level(&mut self, acceleration_level: u16)
    {
        self.config.acceleration_level = acceleration_level.clamp(1, 15);
        self.queue_request(Request::SetAccelerationLevel(self.config.acceleration_level));
    }
    
    pub fn set_deceleration_level(&mut self, deceleration_level: u16)
    {
        self.config.deceleration_level = deceleration_level.clamp(1, 15);
        self.queue_request(Request::SetDecelerationLevel(self.config.deceleration_level));
    }
    
    fn handle_result(&mut self, response: RequestResult)
    {
        match response 
        {
            RequestResult::ReadHoldingRegisters(data) => 
            {
                let frequency_target: u16 = 
                    *data.result.get(0)
                        .unwrap_or(&0);
                        
                let _run_command: u16 = 
                    *data.result.get(1)
                        .unwrap_or(&0);
                        
                let acceleration_level: u16 = 
                    *data.result.get(2)
                        .unwrap_or(&0);
                        
                let deceleration_level: u16 = 
                    *data.result.get(3)
                        .unwrap_or(&0);
                
                self.config.running            = _run_command != 2;
                self.config.frequency_target   = frequency_target;
                self.config.acceleration_level = acceleration_level;
                self.config.deceleration_level = deceleration_level;
                
                self.config.snapshot_id += 1;
            },
            RequestResult::ReadInputRegisters(data) => 
            {
                let frequency: u16 = *data.result.get(InputRegister::CurrentFrequency as usize).unwrap_or(&0);
                
                let voltage: u16 = *data.result.get(InputRegister::BusVoltage as usize).unwrap_or(&0);
                
                let current: u16 = *data.result.get(InputRegister::LineCurrent as usize).unwrap_or(&0);
                
                let temperature: u16 = *data.result.get(InputRegister::DriveTemperature as usize).unwrap_or(&0);
                
                let operation_status: u16 = *data.result.get(InputRegister::SystemStatus as usize).unwrap_or(&0);
                
                self.status = Some(Status {
                    frequency:        units::Frequency::new::<hertz>((frequency as f64) / 10.0),
                    voltage:          units::ElectricPotential::new::<volt>((voltage as f64) / 10.0),
                    current:          units::ElectricCurrent::new::<ampere>((current as f64) / 10.0),
                    temperature:      units::ThermodynamicTemperature::new::<kelvin>((temperature as f64)),
                    operation_status: if operation_status == 0 { OperationStatus::Idle } else if operation_status == 1 {OperationStatus::Running } else { OperationStatus::Fault },
                });
            },
            RequestResult::PresetHoldingRegister(request_result_data) => 
            {
                tracing::error!("PresetHoldingRegister: WORKED");
                
                self.config.snapshot_id += 1;
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
    fn test_requests() {}
}