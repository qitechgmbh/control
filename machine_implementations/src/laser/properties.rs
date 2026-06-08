use qitech_lib::units::{Length, length::millimeter};

use crate::telemetry::{
    BoolProperty, 
    LengthUnit, 
    PropertyManager, 
    LengthProperty
};

#[derive(Debug)]
pub struct Properties {
    pub config: Config,
    pub diameter: LengthProperty,
    pub x_diameter: LengthProperty,
    pub y_diameter: LengthProperty,
    pub in_tolerance: BoolProperty,
    pub global_warning: BoolProperty,
}

impl Properties {
    pub fn new(mgr: &mut PropertyManager) -> Option<Self> {
        let diameter = mgr.create_length_property(
            "diameter", 
            Length::new::<millimeter>(0.0), 
            LengthUnit::Millimeter
        )?;

        let x_diameter = mgr.create_length_property(
            "x_diameter", 
            Length::new::<millimeter>(0.0), 
            LengthUnit::Millimeter
        )?;

        let y_diameter = mgr.create_length_property(
            "y_diameter", 
            Length::new::<millimeter>(0.0), 
            LengthUnit::Millimeter
        )?;

        let target_diameter = mgr.create_length_property(
            "config.target_diameter", 
            Length::new::<millimeter>(0.0), 
            LengthUnit::Millimeter
        )?;

        let higher_tolerance = mgr.create_length_property(
            "config.higher_tolerance", 
            Length::new::<millimeter>(0.0), 
            LengthUnit::Millimeter
        )?;

        let lower_tolerance = mgr.create_length_property(
            "config.lower_tolerance", 
            Length::new::<millimeter>(0.0), 
            LengthUnit::Millimeter
        )?;

        Some(Self { 
            config: Config { 
                target_diameter,
                higher_tolerance,
                lower_tolerance 
            },
            diameter,
            x_diameter,
            y_diameter,
            in_tolerance: mgr.create_bool_property("in_tolerance", false)?, 
            global_warning: mgr.create_bool_property("global_warning", false)?
        })
    }
}

#[derive(Debug)]
pub struct Config {
    target_diameter: LengthProperty,
    higher_tolerance: LengthProperty,
    lower_tolerance: LengthProperty,
}
