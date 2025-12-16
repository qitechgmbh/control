use std::{net::SocketAddr, time::{Duration, Instant}};
use anyhow::Result;
use control_core::{modbus::tcp::ModbusTcpDevice, socketio::{event::{BuildEvent, GenericEvent}, namespace::{CacheFn, CacheableEvents, NamespaceCacheingLogic, cache_duration, cache_first_and_last_event}}};
use control_core_derive::BuildEvent;
use serde::*;
use units::{*, electric_current::milliampere, electric_potential::{millivolt, volt}};
use crate::{MachineChannel, MachineWithChannel};

const MODBUS_DC_OFF: u16 = 0;
const MODBUS_DC_ON: u16 = 1 << 0;
const MODBUS_CONSTANT_POWER: u16 = 1 << 6;
const MODBUS_POWER_PROTECT: u16 = 1 << 9;
const MODBUS_THERMAL_PROTECT: u16 = 1 << 12;

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct LiveValuesEvent {
  voltage: f64,
  current: f64
}

impl CacheableEvents<Self> for LiveValuesEvent {

    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_hours(1), Duration::from_secs(1))
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum Mode {
  Off,
  On24V,
}

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct StateEvent {
    mode: Mode,
    is_default_state: bool,
}

impl CacheableEvents<Self> for StateEvent {

    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

#[derive(Deserialize, Serialize)]
pub enum Mutation {
    TurnOff,
    TurnOn24V,
}

#[derive(Debug)]
pub struct WagoPower {
    mode: Mode,
    channel: MachineChannel,
    device: ModbusTcpDevice,
    last_emit: Instant,
    emitted_default_state: bool,
}

impl WagoPower {

    pub async fn new(channel: MachineChannel, addr: SocketAddr) -> Result<Self> {
        Ok(Self {
            mode: Mode::Off,
            channel,
            device: ModbusTcpDevice::new(addr).await?,
            last_emit: Instant::now(),
            emitted_default_state: false,
        })
    }

    #[cfg(feature = "mock-machine")]
    fn get_live_values(&mut self) -> Result<LiveValuesEvent> {
        match self.mode {
            Mode::Off =>
                Ok(LiveValuesEvent {
                    voltage: 0.0,
                    current: 0.0,
                }),
            Mode::On24V =>
                Ok(LiveValuesEvent {
                    voltage: 24.0,
                    current: 5000.0,
                })
        }
    }

    #[cfg(not(feature = "mock-machine"))]
    fn get_live_values(&mut self) -> Result<LiveValuesEvent> {
        let electric = smol::block_on(
            self.device.get_holding_registers(0x0500, 2)
        )?;

        let voltage = ElectricPotential::new::<millivolt>(f64::from(electric[0]));
        let current = ElectricCurrent::new::<milliampere>(f64::from(electric[1]));

        Ok(LiveValuesEvent {
            voltage: voltage.get::<volt>(),
            current: current.get::<milliampere>(),
        })
    }

    fn emit_state(&mut self) {
        let event = StateEvent {
            mode: self.mode.clone(),
            is_default_state: !self.emitted_default_state,
        };
        self.channel.emit(event);
    }

    fn set_mode(&mut self, mode: Mode) -> Result<()> {
        self.mode = mode;

        #[cfg(not(feature = "mock-machine"))]
        return self.transmit_voltage();

        #[cfg(feature = "mock-machine")]
        return Ok(());
    }

    fn transmit_voltage(&mut self) -> Result<()> {
        let voltage = 24000;
        let current_limit = 0; // For now
        let control_bits = match self.mode {
            Mode::Off => MODBUS_DC_OFF,
            _ => MODBUS_DC_ON | MODBUS_CONSTANT_POWER | MODBUS_THERMAL_PROTECT | MODBUS_POWER_PROTECT,
        };
        let delay_ms = 0; // For now

        let values = vec![voltage, current_limit, control_bits, delay_ms];

        smol::block_on(
            self.device.set_holding_registers(0x0088, &values)
        )
    }
}

impl MachineWithChannel for WagoPower {
    fn get_machine_channel(&self) -> &MachineChannel {
        &self.channel
    }

    fn get_machine_channel_mut(&mut self) -> &mut MachineChannel {
        &mut self.channel
    }

    fn mutate(&mut self, value: serde_json::Value) -> Result<()> {
        let mutation = serde_json::from_value(value)?;

        let mode = match mutation {
            Mutation::TurnOff => Mode::Off,
            Mutation::TurnOn24V => Mode::On24V,
        };

        self.set_mode(mode)
    }

    fn update(&mut self, now: Instant) -> Result<()> {
        if !self.emitted_default_state {
            self.emit_state();

            #[cfg(not(feature = "mock-machine"))]
            self.transmit_voltage()?;

            self.emitted_default_state = true;
         }

        if now - self.last_emit < Duration::from_millis(1000 / 3) {
            if let Ok(event) = self.get_live_values() {
                self.channel.emit(event);
                self.last_emit = now;
            }

            let mode = match self.mode {
                Mode::Off => Mode::On24V,
                _ => Mode::Off,
            };
            self.set_mode(mode)?;
        }

        Ok(())
    }
}
