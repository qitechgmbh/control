use super::{Cooling, WaterCooling, WaterCoolingMode, api::WaterCoolingEvents};
use crate::machines::watercooling::{
    api::WaterCoolingNamespace, temperature_controller::TemperatureController,
};
use anyhow::Error;
use control_core::{actors::digital_output_setter::DigitalOutputSetter, machines::Machine};
use control_core::{
    actors::temperature_input_getter::TemperatureInputGetter,
    machines::{
        identification::DeviceHardwareIdentification,
        new::{
            MachineNewHardware, MachineNewParams, MachineNewTrait,
            get_device_identification_by_role, get_ethercat_device_by_index,
            get_subdevice_by_index, validate_no_role_dublicates,
            validate_same_machine_identification_unique,
        },
    },
};
use ethercat_hal::{
    coe::ConfigurableDevice,
    devices::{
        EthercatDeviceUsed, downcast_device,
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el2008::{EL2008, EL2008_IDENTITY_A, EL2008Port},
        el4002::{EL4002, EL4002_IDENTITY_A, EL4002Configuration},
        subdevice_identity_to_tuple,
    },
    io::{
        analog_output::AnalogOutput, digital_output::DigitalOutput,
        temperature_input::TemperatureInput,
    },
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use uom::si::f64::{AngularVelocity, Pressure, ThermodynamicTemperature};
use uom::si::thermodynamic_temperature::degree_celsius;

impl MachineNewTrait for WaterCooling {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();

        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/WaterCooling::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        smol::block_on(async {
            // Role 0 - Buscoupler EK1100
            {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 0)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WaterCooling::new] Device with role 0 is not Ethercat",
                        module_path!()
                    ))?,
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EK1100_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EK1100>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/WaterCooling::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
            }

            // Role 1 - EL2008 Digital Output Module
            let _el2008 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 1)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WaterCooling::new] Device with role 1 is not Ethercat",
                        module_path!()
                    ))?,
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL2008_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL2008>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/WaterCooling::new] Device with role 1 is not an EL2008",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };

            // Role 2 - EL4002 Analog Output Module
            let _el4002 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 2)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WaterCooling::new] Device with role 2 is not Ethercat",
                        module_path!()
                    ))?,
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL4002_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL4002>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/WaterCooling::new] Device with role 2 is not an el4002",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };
            // Initialize water cooling system
            let water_cooling = Self {
                namespace: WaterCoolingNamespace::new(params.socket_queue_tx.clone()),
                mode: WaterCoolingMode::Standby,
                last_measurement_emit: Instant::now(),
            };

            Ok(water_cooling)
        })
    }
}
