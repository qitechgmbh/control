use anyhow::Error;
use ethercat_hal::{coe::ConfigurableDevice, devices::{ek1100::{EK1100, EK1100_IDENTITY_A}, el2002::{EL2002, EL2002_IDENTITY_A, EL2002_IDENTITY_B, EL2002Port}, el7031::{EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031DigitalInputPort, EL7031StepperPort, coe::EL7031Configuration, pdo::EL7031PredefinedPdoAssignment}, el7031_0030::{self, EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030AnalogInputPort, EL7031_0030StepperPort, coe::EL7031_0030Configuration, pdo::EL7031_0030PredefinedPdoAssignment}, el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port, coe::EL7041_0052Configuration}}, io::{analog_input::AnalogInput, digital_input::DigitalInput, digital_output::DigitalOutput, stepper_velocity_el70x1::StepperVelocityEL70x1}, shared_config::{self, el70x1::{EL70x1OperationMode, StmMotorConfiguration}}};

use crate::{get_ethercat_device, winder2_new::{base::MachineBase, types::Hardware}};
pub use crate::{
    MachineNewHardware, 
    MachineNewParams, 
    MachineNewTrait, 
    validate_no_role_dublicates,
    validate_same_machine_identification_unique,
};

use super::Winder2;

impl MachineNewTrait for Winder2
{
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error>
    {
        let device_identification = params.device_group.to_vec();

        validate_same_machine_identification_unique(&device_identification)?;
        validate_no_role_dublicates(&device_identification)?;

        let hardware = match &params.hardware {
            MachineNewHardware::Ethercat(x) => x,
            _ => {
                return Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] MachineNewHardware is not Ethercat",
                    module_path!()
                ));
            }
        };

        // map the required hardware devices
        let (el2002, el7041, el7031, el7031_0030) = smol::block_on(async 
        {
            // Role 0: Buscoupler EK1100
            let _ek1100 =
                get_ethercat_device::<EK1100>(hardware, params, 0, vec![EK1100_IDENTITY_A]).await?;

            // Role 1: 2x Digital outputs EL2002
            let el2002 = get_ethercat_device::<EL2002>(
                hardware,
                params,
                1,
                vec![EL2002_IDENTITY_A, EL2002_IDENTITY_B],
            ).await?.0;

            // Role 2: Stepper Spool EL7041-0052
            let el7041 = {
                let device = get_ethercat_device::<EL7041_0052>(
                    hardware,
                    params,
                    2,
                    vec![EL7041_0052_IDENTITY_A],
                )
                .await?;

                let el7041_config = EL7041_0052Configuration {
                    stm_features: shared_config::el70x1::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 2800,
                        ..Default::default()
                    },
                    ..Default::default()
                };

                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7041_config)
                    .await?;

                device.0
            };

            // Role 3: Stepper Traverse EL7031
            let el7031 = {
                let device = get_ethercat_device::<EL7031>(
                    hardware,
                    params,
                    3,
                    vec![EL7031_IDENTITY_A, EL7031_IDENTITY_B],
                )
                .await?;

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

            // Role 4: Stepper Puller EL7031-0030
            let el7031_0030 = {
                let device = get_ethercat_device::<EL7031_0030>(
                    hardware,
                    params,
                    4,
                    vec![EL7031_0030_IDENTITY_A],
                )
                .await?;

                let el7031_0030_config = EL7031_0030Configuration {
                    stm_features: el7031_0030::coe::StmFeatures {
                        operation_mode: EL70x1OperationMode::DirectVelocity,
                        speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                        ..Default::default()
                    },
                    stm_motor: StmMotorConfiguration {
                        max_current: 2700,
                        ..Default::default()
                    },
                    pdo_assignment: EL7031_0030PredefinedPdoAssignment::VelocityControlCompact,
                    ..Default::default()
                };
                device
                    .0
                    .write()
                    .await
                    .write_config(&device.1, &el7031_0030_config)
                    .await?;

                device.0
            };

            Ok((el2002, el7041, el7031, el7031_0030))
        })?;

        let machine_id = params
            .device_group
            .first()
            .expect("device group must have at least one device")
            .device_machine_identification
            .machine_identification_unique
            .clone();

        let (sender, receiver) = smol::channel::unbounded();

        let base = MachineBase::new();

        let hardware = Hardware {
            spool_motor: StepperVelocityEL70x1::new(el7041, EL7041_0052Port::STM1),
            traverse_motor: StepperVelocityEL70x1::new(el7031.clone(), EL7031StepperPort::STM1),
            traverse_limit_switch: DigitalInput::new(el7031, EL7031DigitalInputPort::DI1),
            puller_motor: StepperVelocityEL70x1::new(el7031_0030.clone(), EL7031_0030StepperPort::STM1),
            tension_arm_sensor: AnalogInput::new(el7031_0030, EL7031_0030AnalogInputPort::AI1),
            laser: DigitalOutput::new(el2002, EL2002Port::DO1),
        };

        Ok(Self::new(base, hardware))
    }
}