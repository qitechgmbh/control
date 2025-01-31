use std::sync::Arc;
use tokio::sync::RwLock;

pub trait ArcRwLock {
    fn to_arc_rwlock(self) -> Arc<RwLock<Self>>
    where
        Self: Sized,
    {
        Arc::new(RwLock::new(self))
    }
}
