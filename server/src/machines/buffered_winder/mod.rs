use api::{
    BufferedWinderNamespace,
};
use control_core::machines::Machine;
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct BufferedWinder {
    namespace: BufferedWinderNamespace,
    last_measurement_emit: Instant,

}

impl std::fmt::Display for BufferedWinder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferedWinder")
    }
}
impl Machine for BufferedWinder {}

//TODO
