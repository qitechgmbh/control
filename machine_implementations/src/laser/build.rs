use std::time::Instant;
use qitech_lib::{modbus::devices::qitech_laser::LaserDevice, units::{Length, length::millimeter}};
use control_core::machine::{MachineBuild, MachineBuildError, MachineBuilder};
use crate::laser::{DiameterConfig, LaserMachine, api::LaserMachineNamespace};

impl MachineBuild for LaserMachine {
    fn build(mut builder: MachineBuilder<'_>) -> Result<Self, MachineBuildError> {
        let laser = builder.try_get_serial_device_by_index::<LaserDevice>(0)?;

        let diameter_config = DiameterConfig {
            target: builder.config(
                "diameter.target", 
                Length::new::<millimeter>(1.75)
            ).register(),
            tolerance_higher: builder.config(
                "diameter.target", 
                Length::new::<millimeter>(0.05)
            ).register(),
            tolerance_lower: builder.config(
                "diameter.target", 
                Length::new::<millimeter>(0.05)
            ).register(),
        };

        let (sender, receiver) = tokio::sync::mpsc::channel(2);
        let mut laser_machine = Self {
            // --- hardware ---
            laser,

            // --- config ---
            config_diameter: diameter_config,

            // --- measurements
            diameter: builder.measurement("diameter").register(),
            diameter_x: builder.measurement("diameter_x").register(),
            diameter_y: builder.measurement("diametery").register(),
            roundness: builder.measurement("roundness").register(),

            // --- state ---
            last_request: Instant::now(), // not tracked
            in_tolerance: builder.state("in_tolerance").register(),

            // --- events ---
            out_of_tolerance: builder.event("out_of_tolerance", ).register(),

            // --- legacy ---
            error: None,
            last_measurement_emit: Instant::now(),
            namespace: LaserMachineNamespace { namespace: None },
            global_warning: true,
            did_change_state: true,
            emitted_default_state: false,
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: builder.identification(),
        };

        laser_machine.emit_state();
        Ok(laser_machine)
    }
}
