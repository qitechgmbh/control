pub mod registry;
pub mod serial_identification;


pub trait Serial: SerialNew {}

pub trait SerialNew {
    fn new(path: &str) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

#[derive(PartialEq)]
pub struct ProductConfig{
    pub vendor_id: u16,
    pub product_id: u16,
}