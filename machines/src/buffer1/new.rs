use std::time::Instant;

use anyhow::Error;
use ethercat_hal::{
    coe::ConfigurableDevice,
    devices::el7031_0030::coe::EL7031_0030Configuration,
    devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment,
    devices::el7041_0052::coe::EL7041_0052Configuration,
    devices::{
        EthercatDeviceUsed,
        ek1100::{EK1100, EK1100_IDENTITY_A},
        el7031_0030::{EL7031_0030, EL7031_0030_IDENTITY_A},
        el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port},
    },
    io::stepper_velocity_el70x1::StepperVelocityEL70x1,
    shared_config,
    shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration},
};

use crate::{
    MachineNewHardware, MachineNewHardwareEthercat, MachineNewParams, MachineNewTrait,
    buffer1::BufferV1Mode, get_ethercat_device, validate_same_machine_identification_unique,
};
use crate::{buffer1::buffer_tower_controller::BufferTowerController, validate_no_role_dublicates};

use super::{BufferV1, api::Buffer1Namespace};

impl MachineNewTrait for BufferV1 {
    fn new<'maindevice>(
        params: &MachineNewParams<'maindevice, '_, '_, '_, '_, '_, '_>,
    ) -> Result<Self, Error> {
        // validate general stuff
        let device_identification = params.device_group.to_vec();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware: &&MachineNewHardwareEthercat<'_, '_, '_> = match &params.hardware {
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
                .write_config(&subdevice, &el7041_config)
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
                .write_config(&subdevice, &el7031_config)
                .await?;
            {
                let mut device_guard = el7031.write().await;
                device_guard.set_used(true);
            }

            // Controller
            let buffer_tower_controller = BufferTowerController::new(StepperVelocityEL70x1::new(
                el7041.clone(),
                EL7041_0052Port::STM1,
            ));

            let machine_identification_unique = params.get_machine_identification_unique();
            let (sender, receiver) = smol::channel::unbounded();

            // create buffer instance
            let mut buffer: BufferV1 = Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                namespace: Buffer1Namespace {
                    namespace: params.namespace.clone(),
                },
                last_measurement_emit: Instant::now(),
                mode: BufferV1Mode::Standby,
                buffer_tower_controller,
                machine_identification_unique: machine_identification_unique.clone(),
            };
            buffer.emit_state();
            Ok(buffer)
        })
    }
}
