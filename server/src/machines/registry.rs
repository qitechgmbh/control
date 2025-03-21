use super::{new::MachineNewFn, winder1::WinderV1, Machine};
use crate::{
    ethercat::device_identification::{MachineDeviceIdentification, MachineIdentification},
    machines::new::{MACHINE_WINDER_V1, VENDOR_QITECH},
};
use anyhow::Error;
use ethercat_hal::devices::Device;
use ethercrab::{SubDevice, SubDeviceRef};
use lazy_static::lazy_static;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::RwLock;

pub struct MachineRegistry {
    type_map: HashMap<TypeId, (MachineIdentification, MachineNewFn)>,
}

impl MachineRegistry {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn register<T: Machine + 'static>(
        &mut self,
        machine_identficiation: MachineIdentification,
    ) {
        self.type_map.insert(
            TypeId::of::<T>(),
            (
                machine_identficiation,
                Box::new(|identified_device_group, subdevices, devices| {
                    Ok(Arc::new(RwLock::new(T::new(
                        identified_device_group,
                        subdevices,
                        devices,
                    )?)))
                }),
            ),
        );
    }

    pub fn new_machine(
        &self,
        identified_device_group: &Vec<MachineDeviceIdentification>,
        subdevices: &'_ Vec<SubDeviceRef<'_, &SubDevice>>,
        devices: &Vec<Arc<RwLock<dyn Device>>>,
    ) -> Result<Arc<RwLock<dyn Machine>>, anyhow::Error> {
        // get machiine identification
        let machine_identification = &identified_device_group
            .first()
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] No device in group",
                module_path!()
            ))?
            .machine_identification_unique;

        // find machine new function by comparing MachineIdentification
        let (_, machine_new_fn) = self
            .type_map
            .values()
            .find(|(mi, _)| mi == &MachineIdentification::from(machine_identification))
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;

        // call machine new function by reference
        (machine_new_fn)(identified_device_group, subdevices, devices)
    }

    pub fn downcast<T: Machine + 'static>(
        &self,
        machine: Arc<RwLock<dyn Machine>>,
    ) -> Result<Arc<RwLock<T>>, Error> {
        if TypeId::of::<T>() == machine.type_id() {
            // transmute Arc
            let arc = unsafe { Arc::from_raw(Arc::into_raw(machine) as *const RwLock<T>) };
            Ok(arc)
        } else {
            Err(anyhow::anyhow!(
                "[{}::MachineConstructor::downcast] Machine is not of type {}",
                module_path!(),
                std::any::type_name::<T>()
            ))
        }
    }
}

lazy_static! {
    pub static ref MACHINE_REGISTRY: MachineRegistry = {
        let mut mc = MachineRegistry::new();
        mc.register::<WinderV1>(MachineIdentification::new(VENDOR_QITECH, MACHINE_WINDER_V1));
        mc
    };
}
