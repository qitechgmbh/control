use crate::{
    MachineHardware, MachineNew,
    minimal_machines::oversampling_test_machine::{
        AnalogOutOversamplingMachine, ChannelConfig, CYCLE_TIME_US, OVERSAMPLE_FACTOR,
        api::AnalogOutOversamplingNamespace,
    },
};
use anyhow::Error;
use qitech_lib::ethercat_hal::devices::ek1100::EK1100;
use qitech_lib::ethercat_hal::devices::el4732::{
    EL4732, EL4732RxPdo, EL4732_IDENTITY_A, EL4732_IDENTITY_B, EL4732_IDENTITY_C,
};
use std::time::{Duration, Instant};

impl MachineNew for AnalogOutOversamplingMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let interface = hw.ethercat_interface.clone().ok_or_else(|| {
            anyhow::anyhow!("AnalogOutOversamplingMachine: No EtherCAT interface supplied")
        })?;

        // Role 0: EK1100 bus coupler — no configuration needed
        let _ek1100 = hw.try_get_ethercat_device_and_addr_by_role::<EK1100>(0)?;

        // Role 1: EL4732 analog output with oversampling
        let el4732 = hw.try_get_ethercat_device_by_role::<EL4732>(1)?;
        let device_address = hw.try_get_ethercat_meta_by_role(1)?;

        if OVERSAMPLE_FACTOR > 1 {
            // Use configure_oversampling which calls set_oversampling directly on
            // the ethercrab subdevice  bypasses the mailbox/CoE path entirely.
            interface.configure_oversampling(device_address, OVERSAMPLE_FACTOR as u16)?;

            // Rebuild the in-memory RxPdo buffer to match the new PDI size.
            // Without this the byte layout the driver writes won't match what
            // the device expects on the wire.
            el4732.borrow_mut().rxpdo = EL4732RxPdo::new(OVERSAMPLE_FACTOR);
        }

        // Enable DC Sync01 so the device steps between sub-samples at the
        // correct moment. sync1_period = cycle_time / oversample_factor.
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
            last_act: Some(Instant::now()),
        };

        machine.emit_state();
        machine.emit_live_values();
        Ok(machine)
    }
}
