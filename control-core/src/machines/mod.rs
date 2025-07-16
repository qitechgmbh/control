use anyhow::anyhow;
use api::MachineApi;
use new::MachineNewTrait;
use smol::lock::Mutex;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use crate::machines::new::MachineAct;
pub mod api;
pub mod identification;
pub mod manager;
pub mod manager_iter;
pub mod new;
pub mod registry;

pub trait Machine: MachineAct + MachineNewTrait + MachineApi + Any + Debug + Send {
    fn as_any(&self) -> &dyn Any;
}
/// Casts a `dyn Machine` to a specific machine type
pub async fn downcast_machine<T: Machine>(
    machine: Arc<Mutex<dyn Machine>>,
) -> Result<Arc<Mutex<T>>, anyhow::Error> {
    // Acquire a read lock on the RwLock
    let read_lock = machine.lock().await;

    // Check if the inner type can be downcasted to T
    if read_lock.as_any().is::<T>() {
        // Clone the Arc and return it as the desired type
        let cloned_machine = Arc::clone(&machine);
        // Transmute the Arc to the desired type
        unsafe {
            Ok(Arc::from_raw(
                Arc::into_raw(cloned_machine) as *const Mutex<T>
            ))
        }
    } else {
        Err(anyhow!(
            "[{}::downcast_machine] Downcast failed",
            module_path!()
        ))
    }
}
