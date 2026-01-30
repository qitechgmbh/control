use crate::vacuum::api::{VacuumMachineEvents, LiveValuesEvent, StateEvent};
use crate::machine_identification::{MachineIdentification, MachineIdentificationUnique};
use crate::{AsyncThreadMessage, MACHINE_VACUUM, Machine, MachineMessage};
use control_core::socketio::namespace::NamespaceCacheingLogic;
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use serde::{Deserialize, Serialize};
use smol::channel::{Receiver, Sender};
use std::ops::Add;
use std::time::{Duration, Instant};
pub mod act;
pub mod api;
pub mod new;
use crate::vacuum::api::VacuumMachineNamespace;
use crate::{VENDOR_QITECH};

#[derive(Debug)]
pub struct VacuumMachine 
{
    pub api_receiver: Receiver<MachineMessage>,
    pub api_sender: Sender<MachineMessage>,
    pub machine_identification_unique: MachineIdentificationUnique,
    pub namespace: VacuumMachineNamespace,
    pub last_state_emit: Instant,
    pub last_live_values_emit: Instant,
    pub outputs: [bool; 8],
    pub inputs:  [bool; 8],
    pub main_sender: Option<Sender<AsyncThreadMessage>>,
    pub douts: [DigitalOutput; 8],
    pub dins:  [DigitalInput; 8],
    
    pub mode: Mode,
    pub interval_time_off: f64,
    pub interval_time_on:  f64,
    
    // machine state:
    pub interval_state: bool,
    pub interval_expiry: Instant,
    
    pub running: bool,
}


#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[repr(u8)]
pub enum Mode 
{
    Idle,
    On,
    Auto,
    Interval,
}

impl TryFrom<u8> for Mode
{
    type Error = ();

    fn try_from(v: u8) -> Result<Self, Self::Error>
    {
        match v
        {
            0 => Ok(Mode::Idle),
            1 => Ok(Mode::On),
            2 => Ok(Mode::Auto),
            3 => Ok(Mode::Interval),
            _ => Err(()),
        }
    }
}

impl Machine for VacuumMachine 
{
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique 
    {
        self.machine_identification_unique.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> 
    {
        self.main_sender.clone()
    }
}

impl VacuumMachine 
{
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_VACUUM,
    };
}

impl VacuumMachine 
{
    // default functions
    pub fn get_state(&self) -> StateEvent 
    {
        StateEvent {
            mode: self.mode.clone(),
            interval_time_off: self.interval_time_off,
            interval_time_on: self.interval_time_on,
            running: self.running,
        }
    }

    pub fn emit_state(&mut self) 
    {
        let event = self.get_state().build();
        self.namespace.emit(VacuumMachineEvents::State(event));
    }

    pub fn get_live_values(&self) -> LiveValuesEvent 
    {
        LiveValuesEvent 
        {
            remaining_time: match self.mode 
            {
                Mode::Idle => 0.0,
                Mode::On => 0.0,
                Mode::Auto => 999.0,
                Mode::Interval => (self.interval_expiry - Instant::now()).as_secs_f64(),
            }
        }
    }

    pub fn emit_live_values(&mut self) 
    {
        let event = self.get_live_values().build();
        self.namespace
            .emit(VacuumMachineEvents::LiveValues(event));
    }

    fn set_running(&mut self, value: bool)
    {
        if self.running == value { return; }
        self.douts[0].set(value);
        self.running = value;

        tracing::warn!("set running: {}", value);
    }

    // getter/setter for api
    pub fn set_mode(&mut self, value: Mode)
    {
        if self.mode == value { return; }
        
        self.mode = value;
        
        match self.mode 
        {
            Mode::Idle => 
            {
                self.set_running(false);
            },
            Mode::On => 
            {
                self.set_running(true);
            },
            Mode::Auto => 
            {
                //TODO: implement auto mode
                // if self.running { self.douts[0].set(false); }
            },
            Mode::Interval => 
            {
                self.interval_state = true;
                self.interval_expiry = Instant::now().add(Duration::from_secs_f64(self.interval_time_on));
                self.set_running(true);
            },
        }
        
        self.emit_state();
    }
    
    pub fn set_interval_time_off(&mut self, value: f64)
    {
        if self.interval_time_off == value { return; }
        
        self.interval_time_off = value.clamp(1.0, 120.0);
        
        let new_expiry = Instant::now().add(Duration::from_secs_f64(value));

        if self.mode == Mode::Interval && 
           !self.interval_state && 
           new_expiry < self.interval_expiry
        {
            self.interval_expiry = new_expiry;
        }
        
        self.emit_state();
    }
    
    pub fn set_interval_time_on(&mut self, value: f64)
    {
        if self.interval_time_on == value { return; }
        
        self.interval_time_on = value.clamp(1.0, 120.0);
        
        let new_expiry = Instant::now().add(Duration::from_secs_f64(value));

        if self.mode == Mode::Interval && 
           self.interval_state && 
           new_expiry < self.interval_expiry
        {
            self.interval_expiry = new_expiry;
        }
        
        self.emit_state();
    }
}