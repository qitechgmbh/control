use super::{Machine, identification::MachineIdentification, new::MachineNewParams};
use anyhow::Error;
use smol::lock::{Mutex, RwLock};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

pub type MachineNewClosure =
    Box<dyn Fn(&MachineNewParams) -> Result<Box<Mutex<dyn Machine>>, Error> + Send + Sync>;

pub struct MachineRegistry {
    type_map: HashMap<TypeId, (MachineIdentification, MachineNewClosure)>,
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
                // create a machine construction closure
                Box::new(|machine_new_params| {
                    Ok(Box::new(Mutex::new(T::new(machine_new_params)?)))
                }),
            ),
        );
    }

    pub fn new_machine(
        &self,
        machine_new_params: &MachineNewParams,
    ) -> Result<Box<Mutex<dyn Machine>>, anyhow::Error> {
        // get machiine identification
        let device_identification =
            &machine_new_params
                .device_group
                .first()
                .ok_or(anyhow::anyhow!(
                    "[{}::MachineConstructor::new_machine] No device in group",
                    module_path!()
                ))?;

        // find machine new function by comparing MachineIdentification
        let (_, machine_new_closure) = self
            .type_map
            .values()
            .find(|(mi, _)| {
                mi == &device_identification
                    .device_machine_identification
                    .machine_identification_unique
                    .machine_identification
            })
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;

        // call machine new function by reference
        (machine_new_closure)(machine_new_params)
    }

    pub fn downcast_arc_rwlock<T: Machine + 'static>(
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

    pub fn downcast_box_rwlock<T: Machine + 'static>(
        &self,
        machine: Box<RwLock<dyn Machine>>,
    ) -> Result<RwLock<T>, Error> {
        if TypeId::of::<T>() == machine.type_id() {
            // transmute Box
            let box_machine = unsafe { Box::from_raw(Box::into_raw(machine) as *mut RwLock<T>) };
            Ok(*box_machine)
        } else {
            Err(anyhow::anyhow!(
                "[{}::MachineConstructor::downcast] Machine is not of type {}",
                module_path!(),
                std::any::type_name::<T>()
            ))
        }
    }
}
