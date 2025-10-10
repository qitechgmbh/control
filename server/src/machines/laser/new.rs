use std::time::Instant;

use crate::serial::registry::SERIAL_DEVICE_REGISTRY;

#[cfg(not(feature = "laser-mock"))]
use crate::serial::devices::laser::Laser;

#[cfg(feature = "laser-mock")]
use crate::serial::devices::mock_laser::MockLaserDevice;

use super::{LaserMachine, LaserTarget, api::LaserMachineNamespace};
use anyhow::Error;
use control_core::machines::new::{MachineNewHardware, MachineNewTrait};
use uom::ConstZero;
use uom::si::{f64::Length, length::millimeter};

// Helper macro to reduce duplication in downcast logic
macro_rules! downcast_laser_device {
    ($device:expr, $device_type:ty, $error_msg:expr) => {
        match smol::block_on(SERIAL_DEVICE_REGISTRY.downcast_arc_rwlock::<$device_type>($device)) {
            Ok(laser) => laser,
            Err(_) => return Err(Error::msg($error_msg)),
        }
    };
}

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

        // Downcast the hardware_serial to the appropriate device type
        #[cfg(not(feature = "laser-mock"))]
        let laser = downcast_laser_device!(
            hardware_serial.device.clone(),
            Laser,
            "Failed to downcast to Laser"
        );

        #[cfg(feature = "laser-mock")]
        let laser = downcast_laser_device!(
            hardware_serial.device.clone(),
            MockLaserDevice,
            "Failed to downcast to MockLaserDevice"
        );

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
