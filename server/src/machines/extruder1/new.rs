use std::time::Instant;

use super::{ExtruderV2, ExtruderV2Mode, Heating, api::ExtruderV2Namespace};
use anyhow::Error;
use control_core::{
    actors::{
        analog_input_getter::AnalogInputGetter,
        mitsubishi_inverter_rs485::MitsubishiInverterRS485Actor,
    },
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
    devices::{
        downcast_device,
        ek1100::EK1100_IDENTITY_A,
        el2004::EL2004,
        el3021::{EL3021, EL3021_IDENTITY_A, EL3021Port},
        el6021::{self, EL6021, EL6021_IDENTITY_A},
        subdevice_identity_to_tuple,
    },
    io::{
        analog_input::{AnalogInput, physical::AnalogInputRange},
        serial_interface::SerialInterface,
    },
};
use uom::si::electric_current::{ElectricCurrent, milliampere};

impl MachineNewTrait for ExtruderV2 {
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
        };
        // using block_on because making this funciton async creates a lifetime issue
        // if its async the compiler thinks &subdevices is persisted in the future which might never execute
        // so we can't drop subdevices unless this machine is dropped, which is bad
        smol::block_on(async {
            // Role 0
            // Buscoupler
            // EK1100
            {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 0)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                match subdevice_identity_to_tuple(&subdevice_identity) {
                    EK1100_IDENTITY_A => (),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
            }

            let el6021 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 1)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL6021_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL6021>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                }
            };

            let el3021 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 2)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL3021_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL3021>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                }
            };

            let el2004 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat =
                    match &device_identification.device_hardware_identification {
                        DeviceHardwareIdentification::Ethercat(
                            device_hardware_identification_ethercat,
                        ) => device_hardware_identification_ethercat,
                    };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL3021_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL2004>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                }
            };

            let pressure_sensor = AnalogInputGetter::new(AnalogInput::new(el3021, EL3021Port::AI1));

            let extruder: ExtruderV2 = Self {
                inverter: MitsubishiInverterRS485Actor::new(SerialInterface::new(
                    el6021,
                    el6021::EL6021Port::SI1,
                )),
                namespace: ExtruderV2Namespace::new(),
                last_measurement_emit: Instant::now(),
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
