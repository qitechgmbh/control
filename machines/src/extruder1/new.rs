#[cfg(not(feature = "mock-machine"))]
use crate::{MachineNewParams, MachineNewTrait, get_ethercat_device};

#[cfg(not(feature = "mock-machine"))]
use anyhow::Error;

#[cfg(not(feature = "mock-machine"))]
use ethercat_hal::coe::ConfigurableDevice;
#[cfg(not(feature = "mock-machine"))]
use ethercat_hal::devices::{
    EthercatDeviceUsed,
    ek1100::EK1100,
    ek1100::EK1100_IDENTITY_A,
    el1002::EL1002,
    el1002::EL1002_IDENTITY_A,
    el6021::EL6021,
    el6021::EL6021Configuration,
    el6021::EL6021Port,
    el6021::{EL6021_IDENTITY_A, EL6021_IDENTITY_B, EL6021_IDENTITY_C, EL6021_IDENTITY_D},
};
#[cfg(not(feature = "mock-machine"))]
use std::time::Duration;
#[cfg(not(feature = "mock-machine"))]
use std::time::Instant;
#[cfg(not(feature = "mock-machine"))]
use units::angular_velocity::AngularVelocity;
#[cfg(not(feature = "mock-machine"))]
use units::angular_velocity::revolution_per_minute;
#[cfg(not(feature = "mock-machine"))]
use units::pressure::Pressure;
#[cfg(not(feature = "mock-machine"))]
use units::pressure::bar;

#[cfg(not(feature = "mock-machine"))]
use ethercat_hal::{
    devices::{
        el2004::{EL2004, EL2004_IDENTITY_A, EL2004Port},
        el3021::{EL3021, EL3021_IDENTITY_A, EL3021Port},
        el3204::{EL3204, EL3204_IDENTITY_A, EL3204_IDENTITY_B, EL3204Port},
    },
    io::{
        analog_input::AnalogInput, digital_output::DigitalOutput,
        serial_interface::SerialInterface, temperature_input::TemperatureInput,
    },
};
#[cfg(not(feature = "mock-machine"))]
use units::thermodynamic_temperature::{ThermodynamicTemperature, degree_celsius};

#[cfg(not(feature = "mock-machine"))]
use crate::extruder1::temperature_controller::TemperatureController;

#[cfg(not(feature = "mock-machine"))]
use super::{
    ExtruderV2, ExtruderV2Mode, Heating, api::ExtruderV2Namespace, mitsubishi_cs80::MitsubishiCS80,
    screw_speed_controller::ScrewSpeedController,
};

#[cfg(not(feature = "mock-machine"))]
impl MachineNewTrait for ExtruderV2 {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        use crate::{
            MachineNewHardware, MachineNewHardwareEthercat, validate_no_role_dublicates,
            validate_same_machine_identification_unique,
        };

        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware: &&MachineNewHardwareEthercat<'_, '_, '_> = match &params.hardware {
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
            // Role 0 - Buscoupler EK1100

            use control_core::transmission::fixed::FixedTransmission;
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec());

            // What is its use ?
            let _el1002 =
                get_ethercat_device::<EL1002>(hardware, params, 1, [EL1002_IDENTITY_A].to_vec())
                    .await?;

            let el6021 = {
                let identities = [
                    EL6021_IDENTITY_A,
                    EL6021_IDENTITY_B,
                    EL6021_IDENTITY_C,
                    EL6021_IDENTITY_D,
                ]
                .to_vec();
                let device = get_ethercat_device::<EL6021>(hardware, params, 2, identities).await?;

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &EL6021Configuration::default())
                    .await?;
                {
                    let mut device_guard = device.0.write().await;
                    device_guard.set_used(true);
                }
                device.0
            };

            let el2004 =
                get_ethercat_device::<EL2004>(hardware, params, 3, [EL2004_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let el3021 =
                get_ethercat_device::<EL3021>(hardware, params, 4, [EL3021_IDENTITY_A].to_vec())
                    .await?
                    .0;

            let el3204 = get_ethercat_device::<EL3204>(
                hardware,
                params,
                5,
                [EL3204_IDENTITY_A, EL3204_IDENTITY_B].to_vec(),
            )
            .await?
            .0;

            let t1 = TemperatureInput::new(el3204.clone(), EL3204Port::T1);
            let t2 = TemperatureInput::new(el3204.clone(), EL3204Port::T2);
            let t3 = TemperatureInput::new(el3204.clone(), EL3204Port::T3);
            let t4 = TemperatureInput::new(el3204, EL3204Port::T4);

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

            let inverter = MitsubishiCS80::new(SerialInterface::new(el6021, EL6021Port::SI1));

            let target_pressure = Pressure::new::<bar>(0.0);
            let target_rpm = AngularVelocity::new::<revolution_per_minute>(0.0);

            let screw_speed_controller = ScrewSpeedController::new(
                inverter,
                target_pressure,
                target_rpm,
                pressure_sensor,
                FixedTransmission::new(1.0 / 34.0),
            );
            let (sender, receiver) = smol::channel::unbounded();

            let mut extruder: ExtruderV2 = Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: ExtruderV2Namespace {
                    namespace: params.namespace.clone(),
                },
                last_measurement_emit: Instant::now(),
                mode: ExtruderV2Mode::Standby,
                total_energy_kwh: 0.0,
                last_energy_calculation_time: None,
                temperature_controller_front,
                temperature_controller_middle,
                temperature_controller_back,
                temperature_controller_nozzle,
                screw_speed_controller,
                emitted_default_state: false,
                last_status_hash: None,
            };
            extruder.emit_state();
            Ok(extruder)
        })
    }
}
