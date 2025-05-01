use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, RwLock},
};

use anyhow::{Error, Result};

use crate::serial::{
    ProductConfig,
    Serial,
};

pub type SerialNewFn = Box<
    dyn Fn(
            &str
        ) -> Result<Arc<RwLock<dyn Serial>>, Error>
        + Send
        + Sync,
>;

pub struct SerialRegistry {
    type_map: HashMap<TypeId, (ProductConfig, SerialNewFn)>,
}

impl SerialRegistry {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn register<T: Serial + 'static>(
        &mut self,
        machine_identficiation: ProductConfig,
    ) {
        self.type_map.insert(
            TypeId::of::<T>(),
            (
                machine_identficiation,
                Box::new(|path| {
                    Ok(Arc::new(RwLock::new(T::new(
                        path
                    )?)))
                }),
            ),
        );
    }

    pub fn new_machine(
        &self,
        path: &str,
        profuct_config: &ProductConfig,
    ) -> Result<Arc<RwLock<dyn Serial>>, anyhow::Error> {

        // find serial new function by comparing ProdutConfig
        let (_, serial_new_fn) = self
            .type_map
            .values()
            .find(|(pc, _)| pc == profuct_config)
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;

        // call machine new function by reference
        (serial_new_fn)(path)
    }

    pub fn downcast<T: Serial + 'static>(
        &self,
        machine: Arc<RwLock<dyn Serial>>,
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