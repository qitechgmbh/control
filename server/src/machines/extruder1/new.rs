use std::sync::Arc;

use super::{api::ExtruderV2Namespace, ExtruderV2, ExtruderV2Mode, Heating};
use anyhow::Error;
use control_core::{
    actors::{
        analog_input_getter::{AnalogInputGetter, AnalogInputRange},
        mitsubishi_inverter_rs485::MitsubishiInverterRS485Actor,
    },
    identification::MachineDeviceIdentification,
    machines::new::{
        get_device_by_index, get_mdi_by_role, get_subdevice_by_index, validate_no_role_dublicates,
        validate_same_machine_identification, MachineNewTrait,
    },
};
use ethercat_hal::{
    devices::{
        downcast_device,
        ek1100::EK1100_IDENTITY_A,
        el3021::{EL3021Port, EL3021, EL3021_IDENTITY_A},
        el6021::{self, EL6021, EL6021_IDENTITY_A},
        subdevice_identity_to_tuple, Device,
    },
    io::{analog_input::AnalogInput, serial_interface::SerialInterface},
    types::EthercrabSubDevicePreoperational,
};
use smol::lock::RwLock;
use uom::si::{
    electric_current::{milliampere, ElectricCurrent},
    f64::ElectricPotential,
};

impl MachineNewTrait for ExtruderV2 {
    fn new<'maindevice>(
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &Vec<EthercrabSubDevicePreoperational<'maindevice>>,
        devices: &Vec<Arc<RwLock<dyn Device>>>,
    ) -> Result<Self, Error> {
        // validate general stuff
        validate_same_machine_identification(identified_device_group)?;
        validate_no_role_dublicates(identified_device_group)?;

        // using block_on because making this funciton async creates a lifetime issue
        // if its async the compiler thinks &subdevices is persisted in the future which might never execute
        // so we can't drop subdevices unless this machine is dropped, which is bad
        smol::block_on(async {
            // Role 0
            // Buscoupler
            // EK1100
            let mdi = get_mdi_by_role(identified_device_group, 0).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/Winder2::new] No device with role 0",
                module_path!()
            )))?;

            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            match subdevice_identity_to_tuple(&subdevice_identity) {
                EK1100_IDENTITY_A => (),
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Extruder2::new] Device with role 0 is not an EK1100",
                        module_path!()
                    ))
                }
            };

            let mdi = get_mdi_by_role(identified_device_group, 1).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/Extruder2::new] No device with role 1",
                module_path!()
            )))?;

            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();

            let el6021 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL6021_IDENTITY_A => downcast_device::<EL6021>(device.clone()).await?,
                _ => Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Extruder2::new] Device with role 1 is not an EL6021",
                    module_path!()
                ))?,
            };

            let mdi = get_mdi_by_role(identified_device_group, 2).or(Err(anyhow::anyhow!(
                "[{}::MachineNewTrait/Extruder2::new] No device with role 2",
                module_path!()
            )))?;

            let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();

            let el3021 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL3021_IDENTITY_A => downcast_device::<EL3021>(device.clone()).await?,
                _ => Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Extruder2::new] Device with role 2 is not an EL3021",
                    module_path!()
                ))?,
            };

            let pressure_sensor = AnalogInputGetter::new(
                AnalogInput::new(el3021, EL3021Port::AI1),
                AnalogInputRange::Current {
                    min: ElectricCurrent::new::<milliampere>(4.0),
                    max: ElectricCurrent::new::<milliampere>(20.0),
                },
            );

            let extruder: ExtruderV2 = Self {
                inverter: MitsubishiInverterRS485Actor::new(SerialInterface::new(
                    el6021,
                    el6021::EL6021Port::SI1,
                )),
                namespace: ExtruderV2Namespace::new(),
                last_measurement_emit: chrono::Utc::now(),
                pressure_sensor: pressure_sensor,
                mode: ExtruderV2Mode::Standby,
                heating_front: Heating {
                    temperature: 150.0,
                    heating: false,
                    target_temperature: 150.0,
                },
                heating_back: Heating {
                    temperature: 150.0,
                    heating: false,
                    target_temperature: 150.0,
                },
                heating_middle: Heating {
                    temperature: 150.0,
                    heating: false,
                    target_temperature: 150.0,
                },
                uses_rpm: true,
                rpm: 0.0,
                bar: 0.0,
                target_rpm: 0.0,
                target_bar: 0.0,
            };
            Ok(extruder)
        })
    }
}
