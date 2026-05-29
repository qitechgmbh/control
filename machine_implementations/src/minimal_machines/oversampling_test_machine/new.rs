use crate::{
    MachineHardware, MachineNew,
    minimal_machines::oversampling_test_machine::{
        AnalogOutOversamplingMachine, CYCLE_TIME_US, ChannelConfig, OVERSAMPLE_FACTOR, SYNC1_PERIOD_US, api::AnalogOutOversamplingNamespace
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

        let _ek1100 = hw.try_get_ethercat_device_and_addr_by_role::<EK1100>(0)?;

        let el4732 = hw.try_get_ethercat_device_by_role::<EL4732>(1)?;
        let device_address = hw.try_get_ethercat_meta_by_role(1)?;

        if OVERSAMPLE_FACTOR > 1 {
            interface.configure_oversampling(device_address)?;

            el4732.borrow_mut().rxpdo = EL4732RxPdo::new(OVERSAMPLE_FACTOR as usize);
        }

        let sync1_period = Duration::from_micros(SYNC1_PERIOD_US);
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
            last_samples: [[0.0; OVERSAMPLE_FACTOR as usize]; 2],
            last_act: Some(Instant::now()),
        };

        machine.emit_state();
        machine.emit_live_values();
        Ok(machine)
    }
}
