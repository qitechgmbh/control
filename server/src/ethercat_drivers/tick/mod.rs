use std::{future::Future, pin::Pin};

pub trait Tick: Send + Sync {
    fn tick(&mut self, now_ts: u64) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}
