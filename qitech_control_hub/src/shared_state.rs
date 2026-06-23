use std::{collections::HashMap, sync::Arc};

use clickhouse::Client;
use property::ExportedPropertySet;
use tokio::sync::{RwLock, broadcast};


#[derive(Clone)]
pub struct SharedState {
    pub client: Client,
    pub machine_registry: Arc<RwLock<MachineRegistry>>,
    pub snapshot_tx: broadcast::Sender<Arc<ExportedPropertySet>>,
}

pub type MachineRegistry = HashMap<u64, HashMap<String, PropertyType>>;

#[derive(Debug, Clone, Copy)]
pub enum PropertyType {
    Float,
    Integer,
    Boolean,
    String,
}
