use super::api::Winder1Namespace;
use super::linear_spool_speed_controller::LinearSpoolSpeedController;
use super::tension_arm::TensionArm;
use super::{Winder2, Winder2Mode};
use anyhow::Error;
use control_core::actors::analog_input_getter::{AnalogInputGetter, AnalogInputRange};
use control_core::actors::digital_output_setter::DigitalOutputSetter;
use control_core::actors::stepper_driver_el70x1::StepperDriverEL70x1;
use control_core::converters::step_converter::StepConverter;
use control_core::machines::new::{
    MachineNewParams, MachineNewTrait, get_device_by_index, get_mdi_by_role,
    get_subdevice_by_index, validate_no_role_dublicates, validate_same_machine_identification,
};
use ethercat_hal::coe::ConfigurableDevice;
use ethercat_hal::devices::el2002::{EL2002, EL2002Port};
use ethercat_hal::devices::el3001::{
    EL3001, EL3001Configuration, EL3001Port, EL3001PredefinedPdoAssignment,
};
use ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use ethercat_hal::devices::el7041_0052::{EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port};
use ethercat_hal::devices::{downcast_device, subdevice_identity_to_tuple};
use ethercat_hal::devices::{
    ek1100::EK1100_IDENTITY_A, el2002::EL2002_IDENTITY_A, el3001::EL3001_IDENTITY_A,
};
use ethercat_hal::io::analog_input::AnalogInput;
use ethercat_hal::io::digital_output::DigitalOutput;
use ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1;
use ethercat_hal::shared_config::el70x1::{
    EL70x1OperationMode, StmFeatures, StmMotorConfiguration,
};
use uom::si::electric_potential::volt;
use uom::si::f64::ElectricPotential;

impl MachineNewTrait for Winder2 {
    fn new<'maindevice>(params: &MachineNewParams) -> Result<Self, Error> {
        // validate general stuff
        validate_same_machine_identification(params.identified_device_group)?;
        validate_no_role_dublicates(params.identified_device_group)?;

        // using block_on because making this funciton async creates a lifetime issue
        // if its async the compiler thinks &subdevices is persisted in the future which might never execute
        // so we can't drop subdevices unless this machine is dropped, which is bad
        smol::block_on(async {
            // Role 0
            // Buscoupler
            // EK1100
            let mdi =
                get_mdi_by_role(params.identified_device_group, 0).or(Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] No device with role 0",
                    module_path!()
                )))?;
            let subdevice = get_subdevice_by_index(params.subdevices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            match subdevice_identity_to_tuple(&subdevice_identity) {
                EK1100_IDENTITY_A => (),
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 0 is not an EK1100",
                        module_path!()
                    ));
                }
            };

            // Role 1
            // 2x Digitalausgang
            // EL2002
            let mdi =
                get_mdi_by_role(params.identified_device_group, 1).or(Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] No device with role 1",
                    module_path!()
                )))?;
            let subdevice = get_subdevice_by_index(params.subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(params.devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el2002 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL2002_IDENTITY_A => downcast_device::<EL2002>(device.clone()).await?,
                _ => Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] Device with role 1 is not an EL2002",
                    module_path!()
                ))?,
            };

            // Role 2
            // 1x Analogeingang Lastarm
            let mdi =
                get_mdi_by_role(params.identified_device_group, 2).or(Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] No device with role 2",
                    module_path!()
                )))?;
            let subdevice = get_subdevice_by_index(params.subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(params.devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el3001 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL3001_IDENTITY_A => downcast_device::<EL3001>(device.clone()).await?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 2 is not an EL3001",
                        module_path!()
                    ));
                }
            };
            el3001
                .write()
                .await
                .write_config(
                    &subdevice,
                    &EL3001Configuration {
                        pdo_assignment: EL3001PredefinedPdoAssignment::Compact,
                        ..Default::default()
                    },
                )
                .await?;

            // Role 3
            // 1x Stepper Winder
            // EL7041-0052
            let mdi =
                get_mdi_by_role(params.identified_device_group, 3).or(Err(anyhow::anyhow!(
                    "[{}::MachineNewTrait/Winder2::new] No device with role 3",
                    module_path!()
                )))?;
            let subdevice = get_subdevice_by_index(params.subdevices, mdi.subdevice_index)?;
            let device = get_device_by_index(params.devices, mdi.subdevice_index)?;
            let subdevice_identity = subdevice.identity();
            let el7041 = match subdevice_identity_to_tuple(&subdevice_identity) {
                EL7041_0052_IDENTITY_A => downcast_device::<EL7041_0052>(device.clone()).await?,
                _ => {
                    return Err(anyhow::anyhow!(
                        "[{}::MachineNewTrait/Winder2::new] Device with role 3 is not an EL7041-0052",
                        module_path!()
                    ));
                }
            };
            let el7041_config = EL7041_0052Configuration {
                stm_features: StmFeatures {
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

            // Role 4
            // 1x Stepper Traverse
            // EL7031
            // let mdi = get_mdi_by_role(identified_device_group, 4).or(Err(anyhow::anyhow!(
            //     "[{}::MachineNewTrait/Winder2::new] No device with role 4",
            //     module_path!()
            // )))?;
            // let subdevice = get_subdevice_by_index(subdevices, mdi.subdevice_index)?;
            // let device = get_device_by_index(devices, mdi.subdevice_index)?;
            // let subdevice_identity = subdevice.identity();
            // let el7031 = match subdevice_identity_to_tuple(&subdevice_identity) {
            //     EL7041_0052_IDENTITY_A => downcast_device::<EL7041_0052>(device.clone()).await?,
            //     _ => {
            //         return Err(anyhow::anyhow!(
            //             "[{}::MachineNewTrait/Winder2::new] Device with role 4 is not an EL7031",
            //             module_path!()
            //         ))
            //     }
            // };
            // let el7031_config = EL7041_0052Configuration {
            //     stm_features: StmFeatures {
            //         operation_mode: EL70x1OperationMode::DirectVelocity,
            //         ..Default::default()
            //     },
            //     stm_motor: StmMotorConfiguration {
            //         max_current: 1500,
            //         ..Default::default()
            //     },
            //     ..Default::default()
            // };
            // el7031
            //     .write()
            //     .await
            //     .write_config(&subdevice, &el7031_config)
            //     .await?;

            let mut new = Self {
                spool: StepperDriverEL70x1::new(
                    StepperVelocityEL70x1::new(el7041, EL7041_0052Port::STM1),
                    &el7041_config.stm_features.speed_range,
                ),
                tension_arm: TensionArm::new(AnalogInputGetter::new(
                    AnalogInput::new(el3001, EL3001Port::AI1),
                    AnalogInputRange::Potential {
                        min: ElectricPotential::new::<volt>(-10.0),
                        max: ElectricPotential::new::<volt>(10.0),
                    },
                )),
                laser: DigitalOutputSetter::new(DigitalOutput::new(el2002, EL2002Port::DO1)),
                namespace: Winder1Namespace::new(),
                mode: Winder2Mode::Standby,
                spool_step_converter: StepConverter::new(200),
                spool_speed_controller: Box::new(LinearSpoolSpeedController::new(200.0, 1250.0)),
                last_measurement_emit: chrono::Utc::now(),
            };

            // Role 5
            // 1x Stepper Puller
            // EL7031
            // TODO

            // initalize events
            new.emit_traverse_state();
            new.emit_mode_state();
            new.emit_spool_state();
            new.emit_tension_arm_state();

            Ok(new)
        })
    }
}
