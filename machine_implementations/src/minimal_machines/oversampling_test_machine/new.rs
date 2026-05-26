use crate::{
    MachineHardware, MachineNew, minimal_machines::oversampling_test_machine::{AnalogOutOversamplingMachine, CYCLE_TIME_US, ChannelConfig, OVERSAMPLE_FACTOR, api::AnalogOutOversamplingNamespace},
};
use anyhow::Error;
use qitech_lib::ethercat_hal::{
    coe::ConfigurableDevice,
    devices::el4732::{
        EL4732, EL4732_IDENTITY_A, EL4732_IDENTITY_B, EL4732_IDENTITY_C, coe::EL4732Configuration,
    },
};
use std::time::{Duration, Instant};

impl MachineNew for AnalogOutOversamplingMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let interface = hw
            .ethercat_interface
            .clone()
            .ok_or_else(|| anyhow::anyhow!("AnalogOutOversamplingMachine: No EtherCAT interface supplied"))?;

        // Role 0: EL4732 — accept all three hardware revisions
        let el4732: std::rc::Rc<std::cell::RefCell<EL4732>> =
            hw.try_get_ethercat_device_by_role::<EL4732>(0)?;
        let device_address = hw.try_get_ethercat_meta_by_role(0)?;

        // Write the oversampling factor via CoE SDO (must happen in PRE-OP).
        // This also rebuilds the in-memory RxPdo buffer to match the new PDI size.
        let config = EL4732Configuration {
            oversample_factor: OVERSAMPLE_FACTOR,
        };
        el4732
            .borrow_mut()
            .write_config(interface.clone(), device_address, &config)?;

        // Enable DC Sync01 so the device steps between sub-samples at the right moment.
        // sync1_period = cycle_time / oversample_factor
        let sync1_period = Duration::from_micros(CYCLE_TIME_US) / OVERSAMPLE_FACTOR as u32;
        interface.enable_dc_sync01(device_address, sync1_period)?;

        let (sender, receiver) = tokio::sync::mpsc::channel(2);

        let mut machine = Self {
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: hw.identification,
            namespace: AnalogOutOversamplingNamespace { namespace: None },
            last_state_emit: Instant::now(),
            last_live_values_emit: Instant::now(),
            el4732,
            channels: [ChannelConfig::default(), ChannelConfig::default()],
            phase: [0.0; 2],
            last_samples: [[0.0; OVERSAMPLE_FACTOR]; 2],
        };

        machine.emit_state();
        machine.emit_live_values();
        Ok(machine)
    }
}
