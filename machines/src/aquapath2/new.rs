use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};

use super::{
    AquaPathV2, AquaPathV2Mode,
    api::AquaPathV2Namespace,
    controller::{Controller, ControllerConfig},
};
use crate::aquapath1::{Flow, Temperature};
use anyhow::Error;
use ethercat_hal::{
    devices::{
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el2008::{EL2008, EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008Port},
        el3024::{EL3024, EL3024_IDENTITY_A, EL3024Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
        el4002::{EL4002, EL4002_IDENTITY_A, EL4002Port},
    },
    io::{
        analog_input::AnalogInput,
        analog_output::AnalogOutput,
        as008::{As008Flow, As008Temp},
        digital_output::DigitalOutput,
        temperature_input::TemperatureInput,
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

            // Role 1 - EL2008 Digital Output
            let el2008 = get_ethercat_device::<EL2008>(
                hardware,
                params,
                1,
                [EL2008_IDENTITY_A, EL2008_IDENTITY_B].to_vec(),
            )
            .await?
            .0;

            // Role 2 - EL3204 PT100 Temperature Input
            let el3204 = get_ethercat_device::<EL3204>(
                hardware,
                params,
                2,
                [EL3204_IDENTITY_A, EL3204_IDENTITY_B].to_vec(),
            )
            .await?
            .0;

            // Role 3 - EL4002 Analog Output (fan speed)
            let el4002 =
                get_ethercat_device::<EL4002>(hardware, params, 3, [EL4002_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // Role 4 - EL3024 4-channel 4-20mA Analog Input
            // AI1 → front flow (As008Flow), AI2 → front temp (As008Temp)
            // AI3 → back flow (As008Flow),  AI4 → back temp (As008Temp)
            let el3024 =
                get_ethercat_device::<EL3024>(hardware, params, 4, [EL3024_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let front_flow_sensor =
                As008Flow::new(AnalogInput::new(el3024.clone(), EL3024Port::AI1));
            let front_as008_temp =
                As008Temp::new(AnalogInput::new(el3024.clone(), EL3024Port::AI2));
            let back_flow_sensor =
                As008Flow::new(AnalogInput::new(el3024.clone(), EL3024Port::AI3));
            let back_as008_temp = As008Temp::new(AnalogInput::new(el3024.clone(), EL3024Port::AI4));

            // PT100 temperature sensors from EL3204
            let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1); // front, after heating
            let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2); // front, in reservoir
            let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3); // back, after heating
            let t4 = TemperatureInput::new(el3204.clone(), EL3204Port::T4); // back, in reservoir

            // Digital outputs from EL2008
            let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1); // front pump
            let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2); // front heating relay
            let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4); // front cooling relay
            let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5); // back pump
            let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6); // back heating relay
            let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8); // back cooling relay

            // Analog outputs from EL4002 (fan speed control)
            let ao1 = AnalogOutput::new(el4002.clone(), EL4002Port::AO1);
            let ao2 = AnalogOutput::new(el4002.clone(), EL4002Port::AO2);

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
                back_flow_sensor,
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
                front_flow_sensor,
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
                front_as008_temp,
                back_as008_temp,
            };
            machine.emit_state();

            Ok(machine)
        })
    }
}
