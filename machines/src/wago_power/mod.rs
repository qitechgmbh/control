use crate::{MachineChannel, MachineWithChannel};
use anyhow::Result;
use control_core::{
    modbus::tcp::ModbusTcpDevice,
    socketio::{
        event::{BuildEvent, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, NamespaceCacheingLogic, cache_duration,
            cache_first_and_last_event,
        },
    },
};
use control_core_derive::BuildEvent;
use serde::*;
use smol::lock::Mutex;
use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};
use units::{
    electric_current::milliampere,
    electric_potential::{millivolt, volt},
    *,
};

const MODBUS_DC_OFF: u16 = 0;
const MODBUS_DC_ON: u16 = 1;
const MODBUS_HICCUP_POWER: u16 = 1 << 8;

#[derive(Serialize, Debug, Clone, BuildEvent)]
pub struct LiveValuesEvent {
    voltage: f64,
    current: f64,
}

impl CacheableEvents<Self> for LiveValuesEvent {
    fn event_value(&self) -> GenericEvent {
        self.build().into()
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_duration(Duration::from_secs(60 * 60), Duration::from_secs(1))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Mode {
    Off,
    On24V,
}

impl Mode {
    pub fn as_u16(&self) -> u16 {
        match self {
            Mode::Off => MODBUS_DC_OFF,
            Mode::On24V => MODBUS_HICCUP_POWER | MODBUS_DC_ON,
        }
    }
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
    SetMode(Mode),
}

#[derive(Debug)]
pub struct WagoPower {
    mode: Mode,
    channel: MachineChannel,
    #[cfg(not(feature = "mock-machine"))]
    device: Mutex<ModbusTcpDevice>,
    last_emit: Instant,
    emitted_default_state: bool,
}

impl WagoPower {
    pub async fn new(
        channel: MachineChannel,
        #[cfg(not(feature = "mock-machine"))] addr: SocketAddr,
    ) -> Result<Self> {
        Ok(Self {
            mode: Mode::Off,
            channel,
            #[cfg(not(feature = "mock-machine"))]
            device: Mutex::new(ModbusTcpDevice::new(addr).await?),
            last_emit: Instant::now(),
            emitted_default_state: false,
        })
    }

    #[cfg(feature = "mock-machine")]
    fn get_live_values(&mut self) -> Result<LiveValuesEvent> {
        match self.mode {
            Mode::Off => Ok(LiveValuesEvent {
                voltage: 0.0,
                current: 0.0,
            }),
            Mode::On24V => Ok(LiveValuesEvent {
                voltage: 24.0,
                current: 5000.0,
            }),
        }
    }

    #[cfg(not(feature = "mock-machine"))]
    fn get_live_values(&mut self) -> Result<LiveValuesEvent> {
        let electric = smol::block_on(async {
            let mut dev = self.device.lock().await;
            dev.get_holding_registers(0x0500, 2).await
        })?;

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
        smol::block_on(self.transmit_voltage())?;

        self.emit_state();
        Ok(())
    }

    #[cfg(not(feature = "mock-machine"))]
    async fn transmit_voltage(&mut self) -> Result<()> {
        let mut dev = self.device.lock().await;

        let voltage = 24000;
        let warning_threshold = 5000; // For now
        let control_bits = self.mode.as_u16();
        let delay_ms = 100; // For now

        dev.set_holding_registers(
            0x0088,
            &[voltage, warning_threshold, control_bits, delay_ms],
        )
        .await?;

        Ok(())
    }

    #[cfg(feature = "mock-machine")]
    pub async fn get_serial(&mut self) -> Result<u16> {
        Ok(0xbeef)
    }

    #[cfg(not(feature = "mock-machine"))]
    pub async fn get_serial(&mut self) -> Result<u16> {
        let mut dev = self.device.lock().await;
        dev.get_u16(0x000B).await
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
        let mutation: Mutation = serde_json::from_value(value)?;

        match mutation {
            Mutation::SetMode(mode) => self.set_mode(mode)?,
        }

        Ok(())
    }

    fn on_namespace(&mut self) {
        self.emit_state();
    }

    fn update(&mut self, now: Instant) -> Result<()> {
        if !self.emitted_default_state {
            self.set_mode(Mode::Off)?;

            self.emit_state();
            self.emitted_default_state = true;
        }

        if now.duration_since(self.last_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            if let Ok(event) = self.get_live_values() {
                self.channel.emit(event);
            }

            self.last_emit = now;
        }

        Ok(())
    }
}
