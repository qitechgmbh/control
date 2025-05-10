pub mod registry;
use std::any::Any;

pub trait Serial: Any + Send + Sync + SerialNew {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait SerialNew {
    fn new_serial(path: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

#[derive(PartialEq,Clone,Debug)]
pub struct ProductConfig{
    pub vendor_id: u16,
    pub product_id: u16,
}
