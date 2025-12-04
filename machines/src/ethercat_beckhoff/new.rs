use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_dublicates, validate_same_machine_identification_unique,
};
use super::{BeckhoffMachine, MotorState, api::BeckhoffNamespace};
use anyhow::Error;

use ethercat_hal::devices::{
    ek1100::{EK1100, EK1100_IDENTITY_A},
    // WICHTIG: EL7031_IDENTITY_B hier oben mit importieren!
    el7031::{EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031StepperPort},
    el7031::coe::EL7031Configuration,
    el7031::pdo::EL7031PredefinedPdoAssignment,
};
use ethercat_hal::shared_config;
use ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::coe::ConfigurableDevice;

impl MachineNewTrait for BeckhoffMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        let device_identification = params.device_group.iter().cloned().collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => return Err(anyhow::anyhow!("Hardware is not EtherCAT")),
        };

        smol::block_on(async {
            // Role 0: EK1100 (Koppler)
            let _ek1100 = get_ethercat_device::<EK1100>(
                hardware, params, 0, vec![EK1100_IDENTITY_A]
            ).await?;

            // Role 1: EL7031 (Stepper Motor)
            let el7031 = {
                let device = get_ethercat_device::<EL7031>(
                    hardware,
                    params,
                    1,
                    vec![EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                ).await?;

                let el7031_config = EL7031Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_config)
                    .await?;

                device.0
            };

            let motor_driver = StepperVelocityEL70x1::new(
                el7031.clone(),
                EL7031StepperPort::STM1
            );

            let (sender, receiver) = smol::channel::unbounded();

            Ok(Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: BeckhoffNamespace { namespace: params.namespace.clone() },
                motor_driver,
                motor_state: MotorState { enabled: false, target_velocity: 0 },
            })
        })
    }
}