use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    get_subdevice_by_index, validate_no_role_dublicates,
    validate_same_machine_identification_unique,
};

use super::{
    AquaPathV1, AquaPathV1Mode, Flow, Temperature, api::AquaPathV1Namespace, controller::Controller,
};
use anyhow::Error;
use ethercat_hal::{
    coe::ConfigurableDevice,
    devices::{
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el2008::{EL2008, EL2008_IDENTITY_A, EL2008_IDENTITY_B, EL2008Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
        el4002::{EL4002, EL4002_IDENTITY_A, EL4002Port},
        el5152::{
            EL5152, EL5152_IDENTITY_A, EL5152Configuration, EL5152Port,
            EL5152PredefinedPdoAssignment,
        },
    },
    io::{
        analog_output::AnalogOutput, digital_output::DigitalOutput, encoder_input::EncoderInput,
        temperature_input::TemperatureInput,
    },
};
use std::time::{Duration, Instant};
use units::thermodynamic_temperature::{ThermodynamicTemperature, degree_celsius};

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

            // Role 2 - EL4002 Analog Output Module
            let el4002 =
                get_ethercat_device::<EL4002>(hardware, params, 2, [EL4002_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let el3204 = get_ethercat_device::<EL3204>(
                hardware,
                params,
                3,
                [EL3204_IDENTITY_A, EL3204_IDENTITY_B].to_vec(),
            )
            .await?
            .0;

            let el5152 =
                get_ethercat_device::<EL5152>(hardware, params, 4, [EL5152_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let config = EL5152Configuration {
                pdo_assignment: EL5152PredefinedPdoAssignment::Frequency,
                ..Default::default()
            };
            let subdevice = get_subdevice_by_index(hardware.subdevices, 4)?;
            el5152
                .write()
                .await
                .write_config(&subdevice, &config)
                .await?;

            let enc1 = EncoderInput::new(el5152.clone(), EL5152Port::ENC1);

            let enc2 = EncoderInput::new(el5152.clone(), EL5152Port::ENC2);
            //after heating
            let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1);
            //in reservoir
            let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2);
            //after heating
            let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3);
            //in reservoir
            let t4 = TemperatureInput::new(el3204.clone(), EL3204Port::T4);
            //pump flow control
            //phys 1
            let do1 = DigitalOutput::new(el2008.clone(), EL2008Port::DO1);
            //phys 5
            let do2 = DigitalOutput::new(el2008.clone(), EL2008Port::DO2);
            //heating

            //phys 6
            let do4 = DigitalOutput::new(el2008.clone(), EL2008Port::DO4);
            //phys 3
            let do5 = DigitalOutput::new(el2008.clone(), EL2008Port::DO5);
            //phys 7
            let do6 = DigitalOutput::new(el2008.clone(), EL2008Port::DO6);
            //cooling power cut

            //phys 8
            let do8 = DigitalOutput::new(el2008.clone(), EL2008Port::DO8);

            let ao1 = AnalogOutput::new(el4002.clone(), EL4002Port::AO1);
            let ao2 = AnalogOutput::new(el4002.clone(), EL4002Port::AO2);

            let front_controller = Controller::new(
                0.10,
                0.0,
                0.015,
                Duration::from_millis(500),
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                ao1,
                do4,
                do2,
                t1,
                t2,
                Flow::default(),
                do1,
                enc1,
            );

            let back_controller = Controller::new(
                0.10,
                0.0,
                0.015,
                Duration::from_millis(500),
                Temperature::default(),
                ThermodynamicTemperature::new::<degree_celsius>(25.0),
                ao2,
                do8,
                do6,
                t3,
                t4,
                Flow::default(),
                do5,
                enc2,
            );
            let (sender, receiver) = smol::channel::unbounded();
            let mut water_cooling = Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: AquaPathV1Namespace {
                    namespace: params.namespace.clone(),
                },
                mode: AquaPathV1Mode::Standby,
                last_measurement_emit: Instant::now(),
                front_controller,
                back_controller,
            };
            water_cooling.emit_state();

            Ok(water_cooling)
        })
    }
}
