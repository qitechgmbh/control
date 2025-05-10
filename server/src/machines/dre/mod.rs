use smol::lock::RwLock;
use std::{any::Any, sync::Arc};
use anyhow::Error;
use control_core::serial::Serial;


pub mod new;
pub mod act;
pub mod api;


pub struct Dre {
    pub diameter: Arc<RwLock<Result<f32, Error>>>,
    pub path: String,
}

impl Serial for Dre {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}