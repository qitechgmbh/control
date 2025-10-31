use anyhow::anyhow;
use smol::lock::{Mutex, MutexGuard, MutexGuardArc};
use std::any::Any;
use std::sync::Arc;

pub trait Downcast<T> {
    fn downcast(&self) -> Result<T, anyhow::Error>;
}

pub trait DowncastRef<T> {
    fn downcast_ref(&self) -> Result<&T, anyhow::Error>;
}

impl<T: 'static, U: Any + 'static> Downcast<Arc<Mutex<T>>> for Arc<Mutex<U>> {
    fn downcast(&self) -> Result<Arc<Mutex<T>>, anyhow::Error> {
        // Clone the Arc and return it as the desired type
        let cloned: Arc<Mutex<dyn Any>> = self.clone();

        {
            // Acquire a read lock on the RwLock
            let lock: MutexGuardArc<dyn Any> = cloned.lock_arc_blocking();
            tracing::info!("T: {:?}", std::any::type_name::<T>());
            tracing::info!("U: {:?}", std::any::type_name::<U>());
            tracing::info!("Self: {:?}", std::any::type_name_of_val(self));
            tracing::info!("Lock: {:?}", std::any::type_name_of_val(&lock));
            tracing::info!("Lock U{:?}", lock.is::<U>());
            if !lock.is::<MutexGuard<T>>() {
                // Transmute the Arc to the desired type
                return Err(anyhow!("[{}::downcast] Downcast failed", module_path!()));
            }
        }
        unsafe {
            let arc = Arc::from_raw(Arc::into_raw(cloned) as *const Mutex<T>);

            Ok(arc)
        }
    }
}
