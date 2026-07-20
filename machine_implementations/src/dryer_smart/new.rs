use super::DryerSmartMachine;
use super::api::DryerSmartMachineNamespace;
use crate::dryer::device::DryerDevice;
use crate::{MachineHardware, MachineNew};
use anyhow::Error;
use std::time::Instant;

impl MachineNew for DryerSmartMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        let dryer = hw.try_get_serial_device_by_index::<DryerDevice>(0)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(8);

        Ok(Self {
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: hw.identification,
            dryer,
            namespace: DryerSmartMachineNamespace { namespace: None },
            last_emit: Instant::now(),
            received_data: false,
            status: 0,
            temp_process: 0.0,
            temp_safety: 0.0,
            temp_regen_in: 0.0,
            temp_regen_out: 0.0,
            temp_fan_inlet: 0.0,
            temp_return_air: 0.0,
            temp_dew_point: 0.0,
            pwm_fan1: 0.0,
            pwm_fan2: 0.0,
            power_process: 0.0,
            power_regen: 0.0,
            alarm: 0,
            warning: 0,
            target_temperature: 0.0,
            schedule: Default::default(),
            schedule_write_ts: None,
            target_temp_write_ts: None,
            smart_data: Default::default(),
            smart_data_write_ts: None,
        })
    }
}
