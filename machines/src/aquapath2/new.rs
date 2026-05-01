use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

use super::{
    AquaPathV2, AquaPathV2Mode, Flow, Temperature,
    api::AquaPathV2Namespace,
    controller::{Controller, ControllerConfig},
};
use anyhow::Error;
use ethercat_hal::{
    devices::{
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el1124::{EL1124, EL1124_IDENTITY_A, EL1124Port},
        el2008::{EL2008, EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
        el4002::{EL4002, EL4002_IDENTITY_A, EL4002Port},
        el9505::{EL9505, EL9505_IDENTITY_A},
    },
    io::{
        analog_output::AnalogOutput,
        digital_input::DigitalInput,
        digital_output::DigitalOutput,
        temperature_input::TemperatureInput,
        ufm_flow_input::{Ufm02Type, UfmFlowInput},
    },
};
use std::time::Instant;
use units::{
    AngularVelocity,
    angular_velocity::revolution_per_minute,
    thermodynamic_temperature::{ThermodynamicTemperature, degree_celsius},
};

impl MachineNewTrait for AquaPathV2 {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params
            .device_group
            .iter()
            .map(|device_identification| device_identification.clone())
            .collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/AquaPath2::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        smol::block_on(async {
            // Role 0 - Bus Coupler EK1100
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec());

            // Role 1 - EL2008 Digital Output Module
            let el2008 = get_ethercat_device::<EL2008>(
                hardware,
                params,
                1,
                [EL2008_IDENTITY_A, EL2008_IDENTITY_B].to_vec(),
            )
            .await?
            .0;

            // Role 2 - EL3204 Temperature Input Module
            let el3204 = get_ethercat_device::<EL3204>(
                hardware,
                params,
                2,
                [EL3204_IDENTITY_A, EL3204_IDENTITY_B].to_vec(),
            )
            .await?
            .0;

            // Role 3 - EL4002 Analog Output Module
            let el4002 =
                get_ethercat_device::<EL4002>(hardware, params, 3, [EL4002_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // Role 4 - EL9505 Power Supply Terminal (no process data, no configuration needed)
            let _el9505 =
                get_ethercat_device::<EL9505>(hardware, params, 4, [EL9505_IDENTITY_A].to_vec());

            // Role 5 - EL1124 4-Channel Digital Input Module
            let el1124 =
                get_ethercat_device::<EL1124>(hardware, params, 5, [EL1124_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // Flow 1 (front): pulse on DI1, error on DI2
            let flow1_pulse = DigitalInput::new(el1124.clone(), EL1124Port::DI1);
            let flow1_error = DigitalInput::new(el1124.clone(), EL1124Port::DI2);
            let flow1 = UfmFlowInput::new(flow1_pulse, flow1_error, Ufm02Type::default());

            // Flow 2 (back): pulse on DI3, error on DI4
            let flow2_pulse = DigitalInput::new(el1124.clone(), EL1124Port::DI3);
            let flow2_error = DigitalInput::new(el1124.clone(), EL1124Port::DI4);
            let flow2 = UfmFlowInput::new(flow2_pulse, flow2_error, Ufm02Type::default());

            // Temperature sensors
            let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1); // after heating (front)
            let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2); // in reservoir (front)
            let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3); // after heating (back)
            let t4 = TemperatureInput::new(el3204.clone(), EL3204Port::T4); // in reservoir (back)

            // Digital outputs
            let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1); // front pump
            let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2); // front heating relais
            let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4); // front heater relay
            let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5); // front fan
            let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6); // back cooling power cut
            let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8); // back heater relay

            // Analog outputs
            let ao1 = AnalogOutput::new(el4002.clone(), EL4002Port::AO1); // front fan speed
            let ao2 = AnalogOutput::new(el4002.clone(), EL4002Port::AO2); // back fan speed

            let controller_config = ControllerConfig::default();

            let back_controller = Controller::new(
                AquaPathV2::DEFAULT_PID_KP,
                AquaPathV2::DEFAULT_PID_KI,
                AquaPathV2::DEFAULT_PID_KD,
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                ao2,
                do8,
                do6,
                t3,
                t4,
                AngularVelocity::new::<revolution_per_minute>(100.0),
                Flow::default(),
                do5,
                flow2,
                controller_config,
            );

            let front_controller = Controller::new(
                AquaPathV2::DEFAULT_PID_KP,
                AquaPathV2::DEFAULT_PID_KI,
                AquaPathV2::DEFAULT_PID_KD,
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                ao1,
                do4,
                do2,
                t1,
                t2,
                AngularVelocity::new::<revolution_per_minute>(100.0),
                Flow::default(),
                do1,
                flow1,
                controller_config,
            );

            let (sender, receiver) = smol::channel::unbounded();
            let mut machine = Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: AquaPathV2Namespace {
                    namespace: params.namespace.clone(),
                },
                mode: AquaPathV2Mode::Standby,
                ambient_temperature_calibration: ThermodynamicTemperature::new::<degree_celsius>(
                    22.0,
                ),
                last_measurement_emit: Instant::now(),
                front_controller,
                back_controller,
            };
            machine.emit_state();

            Ok(machine)
        })
    }
}
