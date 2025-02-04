use std::sync::Arc;
use tokio::sync::RwLock;

/// Easly produce an Arc<RwLock<Self>> from a type
pub trait ArcRwLock {
    fn to_arc_rwlock(self) -> Arc<RwLock<Self>>
    where
        Self: Sized,
    {
        Arc::new(RwLock::new(self))
    }
    fn to_box(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
