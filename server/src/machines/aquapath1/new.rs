use super::{AquaPathV1, AquaPathV1Mode};
use crate::machines::aquapath1::{
    Flow, Temperature, api::AquaPathV1Namespace, controller::Controller,
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
        el1002::{EL1002, EL1002_IDENTITY_A, EL1002Port},
        el2008::{EL2008, EL2008_IDENTITY_A, EL2008Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
        el4002::{EL4002, EL4002_IDENTITY_A, EL4002Port},
        el5152::{
            EL5152, EL5152_IDENTITY_A, EL5152Configuration, EL5152Port,
            EL5152PredefinedPdoAssignment,
        },
        subdevice_identity_to_tuple,
    },
    io::{
        analog_output::AnalogOutput, digital_input::DigitalInput, digital_output::DigitalOutput,
        temperature_input::TemperatureInput,
    },
};
use std::{
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use uom::si::{
    f64::{ThermodynamicTemperature, VolumeRate},
    thermodynamic_temperature::degree_celsius,
    volume_rate::liter_per_minute,
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
                            "[{}::MachineNewTrait/AquaPath1::new] Device with role 0 is not an EK1100",
                            module_path!()
                        ));
                    }
                };
                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
            }

            // // Role 1 - EL2008 Digital Output Module
            // let el2008 = {
            //     let device_identification =
            //         get_device_identification_by_role(params.device_group, 1)?;
            //     let device_hardware_identification_ethercat = match &device_identification
            //         .device_hardware_identification
            //     {
            //         DeviceHardwareIdentification::Ethercat(
            //             device_hardware_identification_ethercat,
            //         ) => device_hardware_identification_ethercat,
            //         _ => Err(anyhow::anyhow!(
            //             "[{}::MachineNewTrait/AquaPath1::new] Device with role 1 is not Ethercat",
            //             module_path!()
            //         ))?,
            //     };
            //     let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
            //     let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
            //     let subdevice_identity = subdevice.identity();
            //     let device = match subdevice_identity_to_tuple(&subdevice_identity) {
            //         EL2008_IDENTITY_A => {
            //             let ethercat_device = get_ethercat_device_by_index(
            //                 &hardware.ethercat_devices,
            //                 subdevice_index,
            //             )?;
            //             downcast_device::<EL2008>(ethercat_device).await?
            //         }
            //         _ => {
            //             return Err(anyhow::anyhow!(
            //                 "[{}::MachineNewTrait/AquaPath1::new] Device with role 1 is not an EL2008",
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
                        "[{}::MachineNewTrait/AquaPath1::new] Device with role 2 is not an EL4002",
                        module_path!()
                    ))?,
                };

                {
                    let mut device_guard = device.write().await;
                    device_guard.set_used(true);
                }
                device
            };
            let el3204 = {
                let device_identification =
                    get_device_identification_by_role(params.device_group, 3)?;
                let device_hardware_identification_ethercat = match &device_identification
                    .device_hardware_identification
                {
                    DeviceHardwareIdentification::Ethercat(
                        device_hardware_identification_ethercat,
                    ) => device_hardware_identification_ethercat,
                    _ => Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/AquaPath1::new] Device with role 3 is not Ethercat",
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
                            "[{}::MachineNewTrait/AquaPath1::new] Device with role 3 is not an EL3204",
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

            // let el5152 = {
            //     let device_identification =
            //         get_device_identification_by_role(params.device_group, 4)?;
            //     let device_hardware_identification_ethercat = match &device_identification
            //         .device_hardware_identification
            //     {
            //         DeviceHardwareIdentification::Ethercat(
            //             device_hardware_identification_ethercat,
            //         ) => device_hardware_identification_ethercat,
            //         _ => Err(anyhow::anyhow!(
            //             "[{}::MachineNewTrait/AquaPath1::new] Device with role 4 is not Ethercat",
            //             module_path!()
            //         ))?, //uncommented
            //     };

            //     let subdevice_index = device_hardware_identification_ethercat.subdevice_index;
            //     let subdevice = get_subdevice_by_index(hardware.subdevices, subdevice_index)?;
            //     let subdevice_identity = subdevice.identity();
            //     let device = match subdevice_identity_to_tuple(&subdevice_identity) {
            //         EL5152_IDENTITY_A => {
            //             let ethercat_device = get_ethercat_device_by_index(
            //                 &hardware.ethercat_devices,
            //                 subdevice_index,
            //             )?;
            //             downcast_device::<EL5152>(ethercat_device).await?
            //         }
            //         _ => {
            //             return Err(anyhow::anyhow!(
            //                 "[{}::MachineNewTrait/AquaPath1::new] Device with role 4 is not an EL5152",
            //                 module_path!()
            //             ));
            //         }
            //     };
            //     let config = EL5152Configuration {
            //         pdo_assignment: EL5152PredefinedPdoAssignment::Frequency,
            //         ..Default::default()
            //     };
            //     tracing::info!("config");
            //     device
            //         .write()
            //         .await
            //         .write_config(&subdevice, &EL5152Configuration::default())
            //         .await?;

            //     {
            //         let mut device_guard = device.write().await;
            //         device_guard.set_used(true);
            //     }
            //     device
            // };

            //after heating
            let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1);
            //in reservoir
            let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2);
            //after heating
            let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3);
            //in reservoir
            let t4 = TemperatureInput::new(el3204.clone(), EL3204Port::T4);
            //pump flow control
            // //phys 1
            // let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            // //phys 5
            // let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            // //heating
            // //phys 2
            // let do3 = DigitalOutput::new(el2008.clone(), EL2008Port::DO3);
            // //phys 6
            // let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            // //phys 3
            // let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            // //phys 7
            // let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6);
            // //cooling power cut
            // //phys 4
            // let do7 = DigitalOutput::new(el2008.clone(), EL2008Port::DO7);
            // //phys 8
            // let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8);

            let ao1 = AnalogOutput::new(el4002.clone(), EL4002Port::AO1);
            let ao2 = AnalogOutput::new(el4002.clone(), EL4002Port::AO2);

            let front_controller = Controller::new(
                0.16,
                0.0,
                0.008,
                Duration::from_millis(500),
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                ao1,
                // do7,
                // do3,
                // do5,
                t1,
                t2,
                Flow::default(),
                //do1,
                VolumeRate::new::<liter_per_minute>(0.0),
            );

            let back_controller = Controller::new(
                0.16,
                0.0,
                0.008,
                Duration::from_millis(500),
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                ao2,
                // do8,
                // do4,
                // do6,
                t3,
                t4,
                Flow::default(),
                // do2,
                VolumeRate::new::<liter_per_minute>(0.0),
            );
            // let flow_front = Arc::new(RwLock::new(Flow::default()));
            // let flow_back = Arc::new(RwLock::new(Flow::default()));

            // let cooling_controller_front = TemperatureController::new(
            //     0.16,
            //     0.0,
            //     0.008,
            //     Duration::from_millis(500),
            //     Temperature::default(),
            //     flow_front.clone(),
            //     ThermodynamicTemperature::new::<degree_celsius>(35.0),
            //     ao1,
            //     do7,
            //     do3,
            //     do5,
            //     t1,
            //     t2,
            // );

            // let cooling_controller_back = TemperatureController::new(
            //     0.16,
            //     0.0,
            //     0.008,
            //     Duration::from_millis(500),
            //     Temperature::default(),
            //     flow_back.clone(),
            //     ThermodynamicTemperature::new::<degree_celsius>(35.0),
            //     ao2,
            //     do8,
            //     do4,
            //     do6,
            //     t3,
            //     t4,
            // );

            // let mut flow_controller_front = FlowController::new(
            //     //di1,
            //     do1,
            //     VolumeRate::new::<liter_per_minute>(0.0),
            //     flow_front,
            // );
            // let mut flow_controller_back = FlowController::new(
            //     //di2,
            //     do2,
            //     VolumeRate::new::<liter_per_minute>(0.0),
            //     flow_back,
            // );
            // flow_controller_front.update(Instant::now());
            // flow_controller_back.update(Instant::now());
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
                controller: true,
                front_controller,
                back_controller,
            };
            water_cooling.emit_state();

            Ok(water_cooling)
        })
    }
}
