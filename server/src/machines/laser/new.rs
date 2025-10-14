use std::time::Instant;

use crate::serial::{devices::laser::Laser, registry::SERIAL_DEVICE_REGISTRY};

use super::{LaserMachine, LaserTarget, api::LaserMachineNamespace};
use anyhow::Error;
use control_core::machines::connection::MachineCrossConnection;
use control_core::machines::new::{MachineNewHardware, MachineNewTrait};
use uom::ConstZero;
use uom::si::{f64::Length, length::millimeter};

impl MachineNewTrait for LaserMachine {
    fn new(
        params: &control_core::machines::new::MachineNewParams<'_, '_, '_, '_, '_, '_, '_>,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let hardware_serial = match params.hardware {
            MachineNewHardware::Serial(serial) => *serial,
            _ => return Err(Error::msg("Invalid hardware type for LaserMachine")),
        };

        // downcast the hardware_serial to Arc<RwLock<Laser>>

        let laser = match smol::block_on(
            SERIAL_DEVICE_REGISTRY.downcast_arc_rwlock::<Laser>(hardware_serial.device.clone()),
        ) {
            Ok(laser) => laser,
            Err(_) => return Err(Error::msg("Failed to downcast to Laser")),
        };
        // set laser target configuration
        let laser_target = LaserTarget {
            higher_tolerance: Length::new::<millimeter>(0.05),
            lower_tolerance: Length::new::<millimeter>(0.05),
            diameter: Length::new::<millimeter>(1.75),
        };

        let mut laser_machine = Self {
            machine_identification_unique: params.get_machine_identification_unique(),
            laser,
            namespace: LaserMachineNamespace {
                namespace: params.namespace.clone(),
            },
            last_measurement_emit: Instant::now(),
            laser_target,
            emitted_default_state: false,
            machine_manager: params.machine_manager.clone(),
            connected_winder: MachineCrossConnection::new(
                params.machine_manager.clone(),
                &params.get_machine_identification_unique(),
            ),
            diameter: Length::ZERO,
            x_diameter: None,
            y_diameter: None,
            roundness: None,
        };

        // Emit initial state
        laser_machine.emit_state();

        Ok(laser_machine)
    }
}
