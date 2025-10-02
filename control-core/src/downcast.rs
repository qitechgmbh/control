use anyhow::anyhow;
use smol::lock::Mutex;
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
        // Acquire a read lock on the RwLock
        let lock = self.lock_blocking();

        // Check if the inner type can be downcasted to T
        let any: &dyn Any = &*lock;

        if any.is::<T>() {
            // Clone the Arc and return it as the desired type
            let cloned: Arc<Mutex<dyn Any>> = self.clone();

            // Transmute the Arc to the desired type
            unsafe {
                let arc = Arc::from_raw(Arc::into_raw(cloned) as *const Mutex<T>);

                Ok(arc)
            }
        } else {
            Err(anyhow!("[{}::downcast] Downcast failed", module_path!()))
        }
    }
}
