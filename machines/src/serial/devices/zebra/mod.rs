use std::sync::{Arc, atomic::AtomicBool};

mod types;

mod atomic_f64;
use atomic_f64::AtomicF64;

#[derive(Debug)]
pub struct XtremZebra {

    path: String,

    weight: Arc<AtomicF64>,

    // used to indicate other thread to terminate
    shutdown_flag: Arc<AtomicBool>,
}

impl XtremZebra {
    pub fn new(path: String, id: u32) -> aynhow::Result<Self> {



    }
}