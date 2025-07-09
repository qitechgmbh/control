use super::{
    ExtruderV2, ExtruderV2Mode, Heating, api::ExtruderV2Namespace,
    mitsubishi_inverter_rs485::MitsubishiInverterController,
    screw_speed_controller::ScrewSpeedController,
};
use crate::machines::extruder1::temperature_controller::TemperatureController;
use anyhow::Error;
use control_core::machines::{
    identification::DeviceHardwareIdentification,
    new::{
        MachineNewHardware, MachineNewParams, MachineNewTrait, get_device_identification_by_role,
        get_ethercat_device_by_index, get_subdevice_by_index, validate_no_role_dublicates,
        validate_same_machine_identification_unique,
    },
};
use ethercat_hal::{
    coe::ConfigurableDevice,
    devices::{
        EthercatDeviceUsed, downcast_device,
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el1002::{EL1002, EL1002_IDENTITY_A},
        el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port},
        el3021::{EL3021, EL3021_IDENTITY_A, EL3021Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
        el6021::{
            self, EL6021, EL6021_IDENTITY_A, EL6021_IDENTITY_B, EL6021_IDENTITY_C,
            EL6021_IDENTITY_D, EL6021Configuration,
        },
        subdevice_identity_to_tuple,
    },
    io::{
        analog_input::AnalogInput, digital_output::DigitalOutput,
        serial_interface::SerialInterface, temperature_input::TemperatureInput,
    },
};
use std::time::{Duration, Instant};
use uom::si::{
    angular_velocity::revolution_per_minute,
    f64::{AngularVelocity, Pressure, ThermodynamicTemperature},
    pressure::bar,
    thermodynamic_temperature::degree_celsius,
};

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
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Extruder2::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
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
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/ExtruderV2::new] Device with role 0 is not Ethercat",
                        module_path!()
                    ))?, //uncommented
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
                            "[{}::MachineNewTrait/ExtruderV2::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
            }

            // What is its use ?
            let _el1002 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 1)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/ExtruderV2::new] Device with role 1 is not Ethercat",
                        module_path!()
                    ))?, //uncommented
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL1002_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL1002>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/ExtruderV2::new] Device with role 1 is not an EL1002",
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

            let el6021 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 2)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/ExtruderV2::new] Device with role 2 is not Ethercat",
                        module_path!()
                    ))?, //uncommented
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL6021_IDENTITY_A | EL6021_IDENTITY_B | EL6021_IDENTITY_C
                    | EL6021_IDENTITY_D => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL6021>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/ExtruderV2::new] Device with role 2 is not an EL6021",
                            module_path!()
                        ));
                    }
                };
                device
                    .write()
                    .await
                    .write_config(&subdevice, &EL6021Configuration::default())
                    .await?;
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };

            let el2004 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/ExtruderV2::new] Device with role 3 is not Ethercat",
                        module_path!()
                    ))?, //uncommented
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL2004_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL2004>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/ExtruderV2::new] Device with role 3 is not an EL2004",
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

            let el3021 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 4)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/ExtruderV2::new] Device with role 4 is not Ethercat",
                        module_path!()
                    ))?, //uncommented
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL3021_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL3021>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/ExtruderV2::new] Device with role 4 is not an EL3021",
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

            let el3204 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 5)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/ExtruderV2::new] Device with role 5 is not Ethercat",
                        module_path!()
                    ))?, //uncommented
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL3204_IDENTITY_A | EL3204_IDENTITY_B => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL3204>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/ExtruderV2::new] Device with role 5 is not an EL3204",
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

            let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1);
            let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2);
            let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3);
            let t4 = TemperatureInput::new(el3204.clone(), EL3204Port::T4);

            // For the Relais
            let digital_out_1 = DigitalOutput::new(el2004.clone(), EL2004Port::DO1);
            let digital_out_2 = DigitalOutput::new(el2004.clone(), EL2004Port::DO2);
            let digital_out_3 = DigitalOutput::new(el2004.clone(), EL2004Port::DO3);
            let digital_out_4 = DigitalOutput::new(el2004.clone(), EL2004Port::DO4);

            let pressure_sensor = AnalogInput::new(el3021, EL3021Port::AI1);
            // The Extruders temparature Controllers should disable the relais when the max_temperature is reached
            let extruder_max_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);
            // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            let temperature_controller_front = TemperatureController::new(
                0.16,
                0.0,
                0.008,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                extruder_max_temperature,
                t1,
                digital_out_1,
                Heating::default(),
                Duration::from_millis(500),
                700.0,
                1.0,
            );

            // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            let temperature_controller_middle = TemperatureController::new(
                0.16,
                0.0,
                0.008,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                extruder_max_temperature,
                t2,
                digital_out_2,
                Heating::default(),
                Duration::from_millis(500),
                700.0,
                1.0,
            );

            // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            let temperature_controller_back = TemperatureController::new(
                0.16,
                0.0,
                0.008,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                extruder_max_temperature,
                t3,
                digital_out_3,
                Heating::default(),
                Duration::from_millis(500),
                700.0,
                1.0,
            );

            // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            let temperature_controller_nozzle = TemperatureController::new(
                0.16,
                0.0,
                0.008,
                ThermodynamicTemperature::new::<degree_celsius>(150.0),
                extruder_max_temperature,
                t4,
                digital_out_4,
                Heating::default(),
                Duration::from_millis(500),
                200.0,
                0.95,
            );

            let inverter = MitsubishiInverterController::new(SerialInterface::new(
                el6021,
                el6021::EL6021Port::SI1,
            ));

            let target_pressure = Pressure::new::<bar>(0.0);
            let target_rpm = AngularVelocity::new::<revolution_per_minute>(0.0);

            let screw_speed_controller =
                ScrewSpeedController::new(inverter, target_pressure, target_rpm, pressure_sensor);

            let mut extruder: ExtruderV2 = Self {
                namespace: ExtruderV2Namespace::new(params.socket_queue_tx.clone()),
                last_measurement_emit: Instant::now(),
                mode: ExtruderV2Mode::Standby,
                temperature_controller_front: temperature_controller_front,
                temperature_controller_middle: temperature_controller_middle,
                temperature_controller_back: temperature_controller_back,
                temperature_controller_nozzle: temperature_controller_nozzle,
                screw_speed_controller: screw_speed_controller,
                emitted_default_state: false,
            };
            extruder.emit_state();
            Ok(extruder)
        })
    }
}
