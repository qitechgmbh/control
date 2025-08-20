use super::{AquaPathV1, AquaPathV1Mode};
use crate::machines::aquapath1::{
    Cooling,
    api::AquaPathV1Namespace,
    cooling_controller::{self, CoolingController},
    flow_sensor,
};
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
        el2008::{EL2008, EL2008_IDENTITY_A, EL2008Port},
        el3062_0030::{self, EL3062_0030, EL3062_0030_IDENTITY_A, EL3062_0030Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
        el4002::{EL4002, EL4002_IDENTITY_A, EL4002Port, EL4002PredefinedPdoAssignment},
        subdevice_identity_to_tuple,
    },
    io::{
        analog_input::{AnalogInput, AnalogInputDevice},
        analog_output::{AnalogOutput, AnalogOutputDevice, AnalogOutputOutput},
        digital_output::DigitalOutput,
        temperature_input::TemperatureInput,
    },
};
use ethercat_hal::{coe::Configuration, devices::el4002::EL4002Configuration};
use ethercat_hal::{
    devices::el3062_0030::EL3062_0030Configuration,
    shared_config::el40xx::EL40XXChannelConfiguration,
};
use std::time::{Duration, Instant};
use uom::si::{
    electric_potential::volt, f32::ElectricPotential, f64::ThermodynamicTemperature,
    thermodynamic_temperature::degree_celsius,
};

impl MachineNewTrait for AquaPathV1 {
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
                    "[{}::MachineNewTrait/AquaPath1::new] MachineNewHardware is not Ethercat",
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
                        "[{}::MachineNewTrait/AquaPath1::new] Device with role 0 is not Ethercat",
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
            let el2008 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 1)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/AquaPath1::new] Device with role 1 is not Ethercat",
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
            let el4002 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 2)?;

                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/AquaPath1::new] Device with role 2 is not Ethercat",
                        module_path!()
                    ))?,
                };

                let subdevice = get_subdevice_by_index(
                    hardware.subdevices,
                    device_hardware_identification_ethercat.subdevice_index,
                )?;
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice_identity = subdevice.identity();

                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL4002_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL4002>(ethercat_device).await?
                    }
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/WaterCooling::new] Device with role 2 is not an EL4002",
                        module_path!()
                    ))?,
                };

                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };

            // Role 3 - EL3062_0030 Analog Input Module
            let el3062_0030 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/AquaPath::new] Device with role 3 is not Ethercat",
                        module_path!()
                    ))?,
                };
                let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
                let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
                let subdevice_identity = subdevice.identity();
                let device = match subdevice_identity_to_tuple(&subdevice_identity) {
                    EL3062_0030_IDENTITY_A => {
                        let ethercat_device = get_ethercat_device_by_index(
                            &hardware.ethercat_devices,
                            subdevice_index,
                        )?;
                        downcast_device::<EL3062_0030>(ethercat_device).await?
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "[{}::MachineNewTrait/WaterCooling::new] Device with role 3 is not an EL3062_0030",
                            module_path!()
                        ));
                    }
                };
                let config = EL3062_0030Configuration {
                    ..Default::default()
                };
                device
                    .write()
                    .await
                    .write_config(&subdevice, &config)
                    .await?;
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };

            // let el3204 = {
            //     let device_identification =
            //         get_device_identification_by_role(params.device_group, 4)?;
            //     let device_hardware_identification_ethercat = match &device_identification
            //         .device_hardware_identification
            //     {
            //         DeviceHardwareIdentification::Ethercat(
            //             device_hardware_identification_ethercat,
            //         ) => device_hardware_identification_ethercat,
            //         _ => Err(anyhow::anyhow!(
            //             "[{}::MachineNewTrait/ExtruderV2::new] Device with role 4 is not Ethercat",
            //             module_path!()
            //         ))?, //uncommented
            //     };
            //     let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
            //     let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
            //     let subdevice_identity = subdevice.identity();
            //     let device = match subdevice_identity_to_tuple(&subdevice_identity) {
            //         EL3204_IDENTITY_A | EL3204_IDENTITY_B => {
            //             let ethercat_device = get_ethercat_device_by_index(
            //                 &hardware.ethercat_devices,
            //                 subdevice_index,
            //             )?;
            //             downcast_device::<EL3204>(ethercat_device).await?
            //         }
            //         _ => {
            //             return Err(anyhow::anyhow!(
            //                 "[{}::MachineNewTrait/ExtruderV2::new] Device with role 5 is not an EL3204",
            //                 module_path!()
            //             ));
            //         }
            //     };
            //     {
            //         let mut device_guard = device.write().await;
            //         device_guard.set_used(true);
            //     }
            //     device
            // };
            let ao1 = AnalogOutput::new(el4002.clone(), EL4002Port::AO1);
            let ao2 = AnalogOutput::new(el4002.clone(), EL4002Port::AO2);
            let cooling_controller_front = CoolingController::new(
                0.16,
                0.0,
                0.008,
                Duration::from_millis(500),
                Cooling::default(),
                ThermodynamicTemperature::new::<degree_celsius>(10.0),
                ao1,
                1.0,
            );
            let cooling_controller_back = CoolingController::new(
                0.16,
                0.0,
                0.008,
                Duration::from_millis(500),
                Cooling::default(),
                ThermodynamicTemperature::new::<degree_celsius>(10.0),
                ao2,
                1.0,
            );
            // let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1);
            // let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2);
            // let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3);
            // let t4 = TemperatureInput::new(el3204.clone(), EL3204Port::T4);

            // let digital_out_1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            // let digital_out_2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            // let digital_out_3 = DigitalOutput::new(el2008.clone(), EL2008Port::DO3);
            // let digital_out_4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            // let digital_out_5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            // let digital_out_6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6);
            // let digital_out_7 = DigitalOutput::new(el2008.clone(), EL2008Port::DO7);
            // let digital_out_8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8);

            let a1 = AnalogInput::new(el3062_0030.clone(), EL3062_0030Port::AI1);
            let a2 = AnalogInput::new(el3062_0030.clone(), EL3062_0030Port::AI2);
            let mut flow_sensor1 = flow_sensor::FlowSensor::new(a1, 0.0);
            let mut flow_sensor2 = flow_sensor::FlowSensor::new(a2, 0.0);
            flow_sensor1.update(Instant::now());
            flow_sensor2.update(Instant::now());

            // let aquapath_max_temperature = ThermodynamicTemperature::new::<degree_celsius>(100.0);

            // let temperature_controller_front = TemperatureController::new(
            //     0.16,
            //     0.0,
            //     0.008,
            //     ThermodynamicTemperature::new::<degree_celsius>(20.0),
            //     aquapath_max_temperature,
            //     t1,
            //     digital_out_1,
            //     Heating::default(),
            //     Duration::from_millis(500),
            //     700.0,
            //     1.0,
            // );

            // // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            // let temperature_controller_middle = TemperatureController::new(
            //     0.16,
            //     0.0,
            //     0.008,
            //     ThermodynamicTemperature::new::<degree_celsius>(150.0),
            //     extruder_max_temperature,
            //     t2,
            //     digital_out_2,
            //     Heating::default(),
            //     Duration::from_millis(500),
            //     700.0,
            //     1.0,
            // );

            // // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            // let temperature_controller_back = TemperatureController::new(
            //     0.16,
            //     0.0,
            //     0.008,
            //     ThermodynamicTemperature::new::<degree_celsius>(150.0),
            //     extruder_max_temperature,
            //     t3,
            //     digital_out_3,
            //     Heating::default(),
            //     Duration::from_millis(500),
            //     700.0,
            //     1.0,
            // );

            // // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
            // let temperature_controller_nozzle = TemperatureController::new(
            //     0.16,
            //     0.0,
            //     0.008,
            //     ThermodynamicTemperature::new::<degree_celsius>(150.0),
            //     extruder_max_temperature,
            //     t4,
            //     digital_out_4,
            //     Heating::default(),
            //     Duration::from_millis(500),
            //     200.0,
            //     0.95,
            // );

            let mut water_cooling = Self {
                namespace: AquaPathV1Namespace::new(params.socket_queue_tx.clone()),
                mode: AquaPathV1Mode::Standby,
                last_measurement_emit: Instant::now(),
                flow_sensor1,
                flow_sensor2,
                cooling_controller_front,
                cooling_controller_back,
            };
            water_cooling.emit_state();

            Ok(water_cooling)
        })
    }
}
