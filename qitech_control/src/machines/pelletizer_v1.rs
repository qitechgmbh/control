use std::sync::{Arc, RwLock};

pub struct PelletizerV1 
{
    inverter: Arc<RwLock<US3202510>>,
    inverter_snapshot_id: u64,
}

