use std::time::Instant;

use crate::serial::{devices::laser::Laser, registry::SERIAL_DEVICE_REGISTRY};

use super::{LaserMachine, LaserTarget, api::LaserMachineNamespace};
use anyhow::Error;
use control_core::machines::new::MachineNewTrait;
use uom::si::{f64::Length, length::millimeter};

impl MachineNewTrait for LaserMachine {
    fn new<'maindevice, 'subdevices>(
        params: &control_core::machines::new::MachineNewParams<
            'maindevice,
            'subdevices,
            '_,
            '_,
            '_,
            '_,
            '_,
        >,
    ) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let hardware_serial = match params.hardware {
            control_core::machines::new::MachineNewHardware::Serial(serial) => *serial,
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
        Ok(Self {
            laser,
            namespace: LaserMachineNamespace::new(),
            last_measurement_emit: Instant::now(),
            laser_target,
        })
    }
}
