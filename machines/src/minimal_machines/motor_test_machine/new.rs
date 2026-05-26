use super::{MotorState, MotorTestMachine, api::BeckhoffNamespace};
use crate::{
    MachineNewHardware, MachineNewParams, MachineNewTrait, get_ethercat_device,
    validate_no_role_duplicates, validate_same_machine_identification_unique,
};
use anyhow::Error;

use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::ek1100::{EK1100, EK1100_IDENTITY_A};
use ethercat_hal::devices::el7037::coe::EL7037Configuration;
use ethercat_hal::devices::el7037::pdo::EL7037PredefinedPdoAssignment;
use ethercat_hal::devices::el7037::{EL7037, EL7037_IDENTITY_A, EL7037StepperPort};
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config::el70x7::{
    EL70x1OperationMode, EL70x1SpeedRange, EL70x7InfoData, StmFeatures, StmMotorConfiguration,
};

impl MachineNewTrait for MotorTestMachine {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        println!("[{}::new] Creating new MotorTestMachine", module_path!());
        let device_identification = params.device_group.iter().cloned().collect::<Vec<_>>();
        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_duplicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => return Err(anyhow::anyhow!("Hardware is not EtherCAT")),
        };

        smol::block_on(async {
            // Role 0: EK1100 (Koppler)
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, vec![EK1100_IDENTITY_A]).await?;

            // Role 1: EL7037 (Stepper Motor)
            let el7037 = {
                let device = get_ethercat_device::<EL7037>(
                    hardware,
                    params,
                    1,
                    vec![EL7037_IDENTITY_A],
                )
                .await?;

                let el7037_config = EL7037Configuration {
                    stm_features: StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: EL70x1SpeedRange::Steps1000,
                        select_info_data_1: EL70x7InfoData::MotorLoad,
                        select_info_data_2: EL70x7InfoData::MotorDcCurrent,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 1500,
                        ..Default::default()
                    },
                    pdo_assignment: EL7037PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7037_config)
                    .await?;

                device.0
            };

            let motor_driver =
                StepperVelocityEL70x1::new(el7037.clone(), EL7037StepperPort::STM1);

            let (sender, receiver) = smol::channel::unbounded();

            Ok(Self {
                main_sender: params.main_thread_channel.clone(),
                api_receiver: receiver,
                api_sender: sender,
                machine_identification_unique: params.get_machine_identification_unique(),
                namespace: BeckhoffNamespace {
                    namespace: params.namespace.clone(),
                },
                motor_driver,
                motor_state: MotorState {
                    enabled: true,
                    target_velocity: 100,
                },
            })
        })
    }
}
