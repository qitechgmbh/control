use std::{future::Future, pin::Pin};

pub struct DigitalOutput {
    pub write: Box<dyn Fn(bool) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    pub get: Box<dyn Fn() -> Pin<Box<dyn Future<Output = DigitalOutputGet> + Send>> + Send + Sync>,
}

pub struct DigitalOutputGet {
    pub ts: u64,
    pub value: bool,
}
