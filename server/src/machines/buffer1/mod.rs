use api::{
    Buffer1Namespace, Mode,
};
use control_core::machines::Machine;
use std::time::Instant;

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug)]
pub struct Buffer1 {
    namespace: Buffer1Namespace,
    last_measurement_emit: Instant,

    mode: Mode,
}

impl std::fmt::Display for Buffer1 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BufferedWinder")
    }
}
impl Machine for Buffer1 {}

impl Buffer1 {
    //TODO
}
