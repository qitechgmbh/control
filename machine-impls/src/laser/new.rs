use std::time::Instant;

use super::{LaserMachine, api::LaserMachineNamespace};
use crate::{MachineNew, MachineNewArgs, laser::Config};
use anyhow::Error;
use qitech_lib::{
    modbus::devices::qitech_laser::LaserDevice,
    units::{length::millimeter},
};

// include(VAR OUT, "properties/laser_v1.rs");
// property::Diameter
// property::

// generated::properties::laser_v1::;

/*
pub struct Properties {
    diameter: Diameter,
}

impl Properties {
    pub fn 
}

pub struct Diameter {
    inner: SimpleProperty<f64>,
    pub target: PropertyTarget,
}

impl Property for Diameter {
    pub fn init(allocator: &PropertyAllocator) -> Self {

    }

    pub fn sample() {
    }
}

pub struct PropertyTarget {

}

impl PropertyTarget {

}

*/

impl MachineNew for LaserMachine {
    fn new(args: MachineNewArgs) -> Result<Self, Error> {
        let hw = args.hardware;
        let mut props = args.properties;

        println!("building laser machine");
        
        let laser = hw.try_get_serial_device_by_index::<LaserDevice>(0)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(2);

        // let diameter_target = diameter.attach_target(export: bool);
        // let diameter_target = diameter.attach_target_with_tolerance();

        // let properties = args.properties.init::<Properties>();
        // Self { 
        //    diameter: properties.diameter 
        // }

        // initalize properties
        let diameter = props.add_length::<millimeter>("diameter", 0.0, true)?;
        let x_diameter = props.add_length::<millimeter>("x_diameter", 0.0, true)?;
        let y_diameter = props.add_length::<millimeter>("y_diameter", 0.0, true)?;
        
        // diameter_target = diameter.

        let in_tolerance = props.add_bool("in_tolerance", false, false)?;
        let global_warning = props.add_bool("global_warning", false, false)?;

        let target_diameter = 
            props.add_length::<millimeter>("config.target_diameter", 1.75, false)?;

        let higher_tolerance = 
            props.add_length::<millimeter>("config.higher_tolerance", 0.05, false)?;

        let lower_tolerance = 
            props.add_length::<millimeter>("config.lower_tolerance", 0.05, false)?;

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
