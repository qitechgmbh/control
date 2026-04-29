use std::{collections::HashMap, time::Instant};
use anyhow::Error;
use control_core::converters::linear_step_converter::LinearStepConverter;
use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
use qitech_lib::ethercat_hal::devices::el7041_0052::EL7041_0052;
use qitech_lib::ethercat_hal::devices::el7041_0052::coe::EL7041_0052Configuration;
use qitech_lib::ethercat_hal::devices::el7041_0052::pdo::EL7041_0052PredefinedPdoAssignment;
use qitech_lib::ethercat_hal::shared_config;
use qitech_lib::ethercat_hal::shared_config::el70x1::{EL70x1OperationMode, StmMotorConfiguration};
use qitech_lib::units::length::millimeter;
use qitech_lib::units::{Length, Velocity};
use qitech_lib::units::velocity::meter_per_second;
use crate::{MachineHardware, MachineMessage, MachineNew};

use super::api::Sorter1Namespace;
use super::conveyer_belt_controller::ConveyorBeltController;
use super::valve_controller::ValveController;
use super::{Sorter1, Sorter1Mode};

// Required for the new HAL abstraction
use qitech_lib::ethercat_hal::{
    EtherCATThreadChannel, 
    devices::{el2008::EL2008}
};

impl MachineNew for Sorter1 {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        // 1. Fetch devices using the role-based abstraction
        let el7041 = hw.try_get_ethercat_device_by_role::<EL7041_0052>(1)?;
        let el2008 = hw.try_get_ethercat_device_by_role::<EL2008>(2)?;

        // 2. Get EtherCAT interface for configuration writing
        let interface: EtherCATThreadChannel = hw.ethercat_interface
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("[Sorter1] No EtherCAT Interface supplied"))?;

        // 3. Configure Stepper Motor (EL7041)
        let el7041_config = EL7041_0052Configuration {
            stm_features: shared_config::el70x1::StmFeatures {
                operation_mode: EL70x1OperationMode::DirectVelocity,
                speed_range: shared_config::el70x1::EL70x1SpeedRange::Steps1000,
                ..Default::default()
            },
            stm_motor: StmMotorConfiguration {
                max_current: 5000,
                ..Default::default()
            },
            pdo_assignment: EL7041_0052PredefinedPdoAssignment::VelocityControlCompact,
            ..Default::default()
        };

        let mut el7041_ref = el7041.borrow_mut();
        let el7041_address = hw.try_get_ethercat_meta_by_role(1)?;
        el7041_ref.write_config(interface, el7041_address, &el7041_config)?;
        drop(el7041_ref);
        let mode = Sorter1Mode::Standby;
        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);

        let mut new = Self {
            conveyor_belt: el7041,
            air_valve_outputs: el2008,
            air_valve_states: [false; 8],
            conveyor_belt_controller: ConveyorBeltController::new(
                Velocity::new::<meter_per_second>(0.1),
                LinearStepConverter::from_diameter(
                    200,
                    Length::new::<millimeter>(33.3),
                ),
            ),
            valve_controllers: [
                ValveController::new(),
                ValveController::new(),
                ValveController::new(),
                ValveController::new(),
                ValveController::new(),
                ValveController::new(),
                ValveController::new(),
                ValveController::new(),
            ],
            namespace: Sorter1Namespace { namespace: None },
            last_measurement_emit: Instant::now(),
            machine_identification_unique: hw.identification.clone(),
            mode: mode.clone(),
            conveyor_belt_mode: mode.into(),
            emitted_default_state: false,
            scheduled_ejections: HashMap::new(),
            api_receiver: rx,
            api_sender: tx,
        };

        // Initialize state and streams
        new.emit_state();
        Ok(new)
    }
}