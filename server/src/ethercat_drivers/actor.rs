use std::{future::Future, pin::Pin};

pub trait Actor: Send + Sync {
    fn act(&mut self, now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
