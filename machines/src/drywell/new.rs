use super::{DrywellMachine, api::DrywellMachineNamespace};
use crate::serial::{devices::drywell::Drywell, registry::SERIAL_DEVICE_REGISTRY};
use crate::{MachineNewHardware, MachineNewTrait};
use anyhow::Error;
use std::time::Instant;

impl MachineNewTrait for DrywellMachine {
    fn new(params: &crate::MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let hardware_serial = match params.hardware {
            MachineNewHardware::Serial(serial) => *serial,
            _ => return Err(Error::msg("Invalid hardware type for DrywellMachine")),
        };

        let drywell = match smol::block_on(
            SERIAL_DEVICE_REGISTRY.downcast_arc_rwlock::<Drywell>(hardware_serial.device.clone()),
        ) {
            Ok(d) => d,
            Err(_) => return Err(Error::msg("Failed to downcast to Drywell")),
        };

        let (sender, receiver) = smol::channel::unbounded();

        Ok(Self {
            main_sender: params.main_thread_channel.clone(),
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            drywell,
            namespace: DrywellMachineNamespace {
                namespace: params.namespace.clone(),
            },
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
            last_emit: Instant::now(),
        })
    }
}
