use std::{ time::Instant};

use anyhow::Error;
use qitech_lib::{modbus::{ devices::qitech_laser::LaserDevice, managers::example_manager::ExampleScheduler}, units::{ConstZero, Length, length::millimeter}};
use crate::{MachineHardware, MachineNew};
use super::{LaserMachine, LaserTarget, api::LaserMachineNamespace};

impl MachineNew for LaserMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {        
        
        let laser = hw.try_get_serial_device_by_index::<LaserDevice<ExampleScheduler>>(0)?;
        let mgr = hw.try_get_modbus_mgr_by_index(0)?;

        let laser_target = LaserTarget {
            higher_tolerance: Length::new::<millimeter>(0.05),
            lower_tolerance: Length::new::<millimeter>(0.05),
            diameter: Length::new::<millimeter>(1.75),
        };
        println!("Hello WOrld");
        let (sender,receiver) = tokio::sync::mpsc::channel(2);
        let mut laser_machine = Self {
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: hw.identification,
            mutation_counter: 0,
            laser,
            namespace: LaserMachineNamespace {
                namespace: None,
            },
            last_measurement_emit: Instant::now(),
            laser_target,
            emitted_default_state: false,
            diameter: Length::ZERO,
            target_diameter: Length::ZERO,
            x_diameter: None,
            y_diameter: None,
            roundness: None,
            lower_tolerance: Length::new::<millimeter>(0.05),
            higher_tolerance: Length::new::<millimeter>(0.05),
            in_tolerance: true,
            global_warning: true,
            did_change_state: true,
            modbus_mgr: mgr,
            laser_state: crate::laser::LaserRequestState::NotWaiting,
        };
        laser_machine.emit_state();
        Ok(laser_machine)
    }

}
