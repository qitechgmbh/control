mod winder2_imports {
    pub use super::super::api::Winder2Namespace;
    pub use super::super::tension_arm::TensionArm;
    pub use super::super::{Winder2, Winder2Mode};
    pub use crate::winder2::puller_speed_controller::PullerSpeedController;
    pub use crate::winder2::spool_speed_controller::SpoolSpeedController;
    pub use crate::winder2::traverse_controller::TraverseController;
    pub use anyhow::Error;
    pub use control_core::converters::angular_step_converter::AngularStepConverter;
    pub use control_core::converters::linear_step_converter::LinearStepConverter;

    pub use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
    pub use qitech_lib::ethercat_hal::devices::ek1100::EK1100;
    pub use qitech_lib::ethercat_hal::devices::el2002::{EL2002, EL2002_IDENTITY_B, EL2002Port};
    pub use qitech_lib::ethercat_hal::devices::el7031::coe::EL7031Configuration;
    pub use qitech_lib::ethercat_hal::devices::el7031::pdo::EL7031PredefinedPdoAssignment;
    pub use qitech_lib::ethercat_hal::devices::el7031::{
        EL7031, EL7031_IDENTITY_A, EL7031_IDENTITY_B, EL7031DigitalInputPort, EL7031StepperPort,
    };
    pub use qitech_lib::ethercat_hal::devices::el7031_0030::coe::EL7031_0030Configuration;
    pub use qitech_lib::ethercat_hal::devices::el7031_0030::pdo::EL7031_0030PredefinedPdoAssignment;
    pub use qitech_lib::ethercat_hal::devices::el7031_0030::{
        self, EL7031_0030, EL7031_0030_IDENTITY_A, EL7031_0030AnalogInputPort,
        EL7031_0030StepperPort,
    };
    pub use qitech_lib::ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
    pub use qitech_lib::ethercat_hal::devices::el7041_0052::{
        EL7041_0052, EL7041_0052_IDENTITY_A, EL7041_0052Port,
    };
    pub use qitech_lib::ethercat_hal::devices::{ek1100::EK1100_IDENTITY_A, el2002::EL2002_IDENTITY_A};
    pub use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
    pub use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;
    pub use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;

    pub use qitech_lib::ethercat_hal::shared_config;
    pub use qitech_lib::ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
    pub use std::time::Instant;
    pub use qitech_lib::units::ConstZero;
    pub use qitech_lib::units::f64::*;
    pub use qitech_lib::units::length::{centimeter, meter, millimeter};
    pub use qitech_lib::units::velocity::meter_per_minute;
}

use std::{cell::RefCell, rc::Rc};

use qitech_lib::ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1Device;
pub use winder2_imports::*;
use crate::{MachineHardware, MachineNew};

impl MachineNew for Winder2 {
    fn new(hw: MachineHardware) -> Result<Self, Error> {        
        let _ek1100 = hw.try_get_ethercat_device_by_role::<EK1100>(0)?;
        let el2002 : Rc<RefCell<dyn DigitalOutputDevice>> = hw.try_get_ethercat_device_by_role::<EL2002>(1)?;
        let el7041 : Rc<RefCell<dyn StepperVelocityEL70x1Device>> = hw.try_get_ethercat_device_by_role::<EL7041_0052>(2)?;
        let el7031 : Rc<RefCell<dyn StepperVelocityEL70x1Device>> = hw.try_get_ethercat_device_by_role::<EL7031>(3)?;
        let el7031_0030 : Rc<RefCell<dyn StepperVelocityEL70x1Device>> = hw.try_get_ethercat_device_by_role::<EL7031_0030>(4)?;

        let mode = Winder2Mode::Standby;
        let (sender,receiver) = tokio::sync::mpsc::channel(2);
        
        let mut new = Self {
            api_receiver: receiver,
            api_sender: sender,
            traverse: el7031,            
            puller: el7031_0030.clone(),
            spool: el7041,

            tension_arm: TensionArm::new(el7031_0030.clone()),
            laser: el2002,
            namespace: Winder2Namespace {
                namespace: None,
            },
            mode: mode.clone(),
            spool_step_converter: AngularStepConverter::new(200),
            spool_speed_controller: SpoolSpeedController::new(),
            last_measurement_emit: Instant::now(),
            spool_mode: mode.clone().into(),
            traverse_mode: mode.clone().into(),
            puller_mode: mode.into(),
            puller_speed_controller: PullerSpeedController::new(
                Velocity::new::<meter_per_minute>(1.0),
                LinearStepConverter::from_diameter(
                    200,                            // Assuming 200 steps per revolution for the puller stepper,
                    Length::new::<centimeter>(8.0), // 8cm diameter of the puller wheel
                ),
            ),
            puller_reference_machine: None,
            traverse_controller: TraverseController::new(
                Length::new::<millimeter>(22.0), // Default inner limit
                Length::new::<millimeter>(92.0), // Default outer limit
                64,                              // Microsteps
            ),
            emitted_default_state: false,
            spool_automatic_action: super::SpoolAutomaticAction {
                progress: Length::ZERO,
                progress_last_check: Instant::now(),
                target_length: Length::new::<meter>(250.0),
                mode: super::api::SpoolAutomaticActionMode::NoAction,
            },
            machine_identification_unique: hw.identification,
            laser_enabled: false,
        };

        // initalize events
            new.emit_state();
            Ok(new)
        
    }
}
