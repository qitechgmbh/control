use super::{Machine, identification::MachineIdentification, new::MachineNewParams};
use anyhow::Error;
use smol::lock::Mutex;
use std::{any::TypeId, collections::HashMap};

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
}
