use std::time::Instant;

use crate::serial::{devices::laser::Laser, registry::SERIAL_DEVICE_REGISTRY};
use crate::{MachineNewHardware, MachineNewTrait};

use super::{LaserMachine, LaserTarget, api::LaserMachineNamespace};
use anyhow::Error;
use units::ConstZero;
use units::length::{Length, millimeter};

impl MachineNewTrait for LaserMachine {
    fn new(params: &crate::MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error>
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
        let (sender, receiver) = smol::channel::unbounded();

        let laser_machine = Self {
            main_sender: params.main_thread_channel.clone(),
            api_receiver: receiver,
            api_sender: sender,
            machine_identification_unique: params.get_machine_identification_unique(),
            laser,
            namespace: LaserMachineNamespace {
                namespace: params.namespace.clone(),
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
            did_change_state: true,
        };

        Ok(laser_machine)
    }
}
