use std::time::Instant;

use anyhow::Error;
use control_core::converters::linear_step_converter::LinearStepConverter;
use control_core::machines::connection::MachineCrossConnection;
use control_core::machines::new::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, validate_no_role_dublicates,
    validate_same_machine_identification_unique,
};
use control_core::uom_extensions::velocity::meter_per_minute;
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::EthercatDeviceUsed;
use ethercat_hal::devices::ek1100::EK1100;
use ethercat_hal::devices::ek1100::EK1100_IDENTITY_A;
use ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
use ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
use ethercat_hal::devices::el7031_0030::{
    EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030StepperPort,
};
use ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use ethercat_hal::devices::el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port};
use ethercat_hal::io::digital_input::DigitalInput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
use uom::si::f64::{Length, Velocity};
use uom::si::length::{centimeter, millimeter};

use crate::machines::buffer1::BufferV1Mode;
use crate::machines::buffer1::buffer_lift_controller::BufferLiftController;
use crate::machines::buffer1::puller_speed_controller::PullerSpeedController;
use crate::machines::get_ethercat_device;

use super::{BufferV1, api::Buffer1Namespace};

impl MachineNewTrait for BufferV1 {
    fn new<'maindevice>(
        params: &MachineNewParams<'maindevice, '_, '_, '_, '_, '_, '_>,
    ) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware: &&control_core::machines::new::MachineNewHardwareEthercat<'_, '_, '_> =
            match &params.hardware {
                MachineNewHardware::Ethercat(x) => x,
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Buffer::new] MachineNewHardware is not Ethercat",
                        module_path!()
                    ));
                }
            };

        smol::block_on(async {
            // Role 0 - Buscoupler EK1100
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, [EK1100_IDENTITY_A].to_vec())
                    .await?
                    .0;

            // Role 1 - Stepper Buffer EL7041-0052
            let (el7041, subdevice) = get_ethercat_device::<EL7041_0052>(
                hardware,
                params,
                1,
                [EL7041_0052_IDENTITY_A].to_vec(),
            )
            .await?;

            let el7041_config = EL7041_0052Configuration {
                stm_features: shared_config::el70x1::StmFeatures {
                    operation_mode: EL70x1OperationMode::DirectVelocity,
                    ..Default::default()
                },
                stm_motor: StmMotorConfiguration {
                    max_current: 6000,
                    ..Default::default()
                },
                ..Default::default()
            };

            el7041
                .write()
                .await
                .write_config(subdevice, &el7041_config)
                .await?;
            {
                let mut device_guard = el7041.write().await;
                device_guard.set_used(true);
            }

            // Role 2 - Stepper Puller EL7031-0030
            let (el7031, subdevice) = get_ethercat_device::<EL7031_0030>(
                hardware,
                params,
                2,
                [EL7031_0030_IDENTITY_A].to_vec(),
            )
            .await?;

            let el7031_config = EL7031_0030Configuration {
                stm_features: ethercat_hal::devices::el7031_0030::coe::StmFeatures {
                    operation_mode: EL70x1OperationMode::DirectVelocity,
                    // Max Speed of 1000 steps/s
                    // Max @ 8cm diameter = approx 75 m/min
                    speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                    ..Default::default()
                },
                stm_motor: StmMotorConfiguration {
                    max_current: 1500,
                    ..Default::default()
                },
                pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                ..Default::default()
            };

            el7031
                .write()
                .await
                .write_config(subdevice, &el7031_config)
                .await?;
            {
                let mut device_guard = el7031.write().await;
                device_guard.set_used(true);
            }

            // Controller
            let buffer_lift_controller = BufferLiftController::new(
            let buffer_lift_controller = BufferLiftController::new(
                StepperVelocityEL70x1::new(el7041.clone(), EL7041_0052Port::STM1),
                Length::new::<centimeter>(135.0),
                Length::new::<centimeter>(0.0),
                64,
            );
            let puller_speed_controller = PullerSpeedController::new(
                Velocity::new::<meter_per_minute>(1.0),
                Length::new::<millimeter>(1.75),
                LinearStepConverter::from_diameter(200, Length::new::<centimeter>(8.0)),
            );

            let machine_identification_unique = params.get_machine_identification_unique();

            // create buffer instance
            let mut buffer = Self {
                lift: StepperVelocityEL70x1::new(el7041.clone(), EL7041_0052Port::STM1),
                lift_end_stop: DigitalInput::new(el7041, EL7041_0052Port::DI1),
                puller: StepperVelocityEL70x1::new(el7031.clone(), EL7031_0030StepperPort::STM1),
                lift_step_converter: LinearStepConverter::from_diameter(
                    200,
                    Length::new::<millimeter>(32.22),
                ),
                namespace: Buffer1Namespace::new(params.socket_queue_tx.clone()),
                last_measurement_emit: Instant::now(),
                mode: BufferV1Mode::Standby,
                buffer_lift_controller,
                puller_speed_controller,
                machine_manager: params.machine_manager.clone(),
                machine_identification_unique: machine_identification_unique.clone(),
                connected_winder: MachineCrossConnection::new(
                    params.machine_manager.clone(),
                    &machine_identification_unique,
                ),
            };
            buffer.emit_state();
            Ok(buffer)
        })
    }
}
