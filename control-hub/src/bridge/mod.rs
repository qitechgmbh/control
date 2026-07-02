use std::{collections::HashSet, io};

use machine_core::property::PropertyBatch;

use crate::SharedState;

// pub mod local;
pub mod unix;
pub mod embedded;

pub fn update_registry(batch: &PropertyBatch, registry: &mut HashSet<u64>) -> bool {
    let mut changed = false;

    for prop in &batch.floats {
        changed |= registry.insert(prop.ident);
    }

    for prop in &batch.integers {
        changed |= registry.insert(prop.ident);
    }

    changed
}

pub trait Bridge
where
    Self: Send + 'static,
{
    fn run(self, state: SharedState) -> impl Future<Output = io::Result<()>> + Send;
}