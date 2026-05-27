use std::time::Instant;

use super::{LaserMachine, LaserTarget, api::LaserMachineNamespace};
use crate::{MachineHardware, MachineNew};
use anyhow::Error;
use qitech_lib::{
    modbus::devices::qitech_laser::LaserDevice,
    units::{ConstZero, Length, length::millimeter},
};

impl MachineNew for LaserMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        println!("building laser machine");
        let laser = hw.try_get_serial_device_by_index::<LaserDevice>(0)?;
        let laser_target = LaserTarget {
            higher_tolerance: Length::new::<millimeter>(0.05),
            lower_tolerance: Length::new::<millimeter>(0.05),
            diameter: Length::new::<millimeter>(1.75),
        };

        let (sender, receiver) = tokio::sync::mpsc::channel(2);
        let mut laser_machine = Self {
            error: None,
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: hw.identification,
            mutation_counter: 0,
            laser,
            namespace: LaserMachineNamespace { namespace: None },
            last_measurement_emit: Instant::now(),
            last_request: Instant::now(),
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
        };
        laser_machine.emit_state();
        Ok(laser_machine)
    }
}
