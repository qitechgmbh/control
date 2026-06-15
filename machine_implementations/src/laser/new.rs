use std::time::Instant;

use super::{LaserMachine, api::LaserMachineNamespace};
use crate::{MachineNew, MachineNewArgs, laser::Config};
use anyhow::Error;
use qitech_lib::{
    modbus::devices::qitech_laser::LaserDevice,
    units::{length::millimeter},
};

impl MachineNew for LaserMachine {
    fn new(args: MachineNewArgs) -> Result<Self, Error> {
        let hw = args.hardware;
        let mut props = args.properties;

        println!("building laser machine");
        
        let laser = hw.try_get_serial_device_by_index::<LaserDevice>(0)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(2);

        // initalize properties
        let diameter = props.add_length::<millimeter>("diameter", 0.0)?;
        let x_diameter = props.add_length::<millimeter>("x_diameter", 0.0)?;
        let y_diameter = props.add_length::<millimeter>("y_diameter", 0.0)?;
        
        let in_tolerance = props.add_bool("input_tolerance", false)?;
        let global_warning = props.add_bool("global_warning", false)?;

        let target_diameter = 
            props.add_length::<millimeter>("config.target_diameter", 1.75)?;

        let higher_tolerance = 
            props.add_length::<millimeter>("config.higher_tolerance", 0.05)?;

        let lower_tolerance = 
            props.add_length::<millimeter>("config.lower_tolerance", 0.05)?;

        let config = Config {
            target_diameter,
            higher_tolerance,
            lower_tolerance,
        };

        let mut laser_machine = Self {
            error: None,
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: args.ident,
            laser,
            namespace: LaserMachineNamespace { namespace: None },
            last_measurement_emit: Instant::now(),
            last_request: Instant::now(),
            emitted_default_state: false,
            did_change_state: false,
            // properties
            config,
            diameter,
            x_diameter,
            y_diameter,
            in_tolerance,
            global_warning,
        };

        laser_machine.emit_state();
        Ok(laser_machine)
    }
}
