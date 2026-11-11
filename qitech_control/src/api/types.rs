use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use arc_swap::ArcSwap;
use qitech_framework::ConfigPropertyEventRecord;
use qitech_framework::Constraints;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::MachineSchema;
use qitech_framework::ScalarValue;
use qitech_framework::StatePropertyEventRecord;
use qitech_framework::machine::OperationCapability;

#[derive(Default, Clone)]
pub struct SharedState {
    pub schemas: Swappable<HashMap<MachineIdentification, MachineSchema>>,
    // pub machines: Swappable<HashMap<MachineInstanceIdentification, MachineInstance>>,
}

#[derive(Default, Clone)]
pub struct MachineInstance {
    pub config_properties: HashMap<String, Option<ConfigPropertyInfo>>,
    pub state_properties: HashMap<String, Option<StatePropertyInfo>>,
    pub measurements: HashMap<String, Option<MeasurementInfo>>,

    // --- misc ---
    pub subscriptions: HashSet<MachineInstanceIdentification>,
}

#[derive(Clone)]
pub struct ConfigPropertyInfo {
    pub value: ScalarValue,
    pub default: ScalarValue,
    pub capability: OperationCapability,
    pub constraints: Constraints,

    #[allow(unused)]
    pub records: Vec<ConfigPropertyEventRecord>,
}

#[derive(Clone)]
pub struct StatePropertyInfo {
    pub value: ScalarValue,

    #[allow(unused)]
    pub records: Vec<StatePropertyEventRecord>,
}

#[derive(Clone)]
pub struct MeasurementInfo {
    pub value: Option<f64>,
}

/// Lock-free reads, serialized writes.
#[derive(Default, Clone)]
pub struct Swappable<T: Clone> {
    value: Arc<ArcSwap<T>>,
    // --- writers do load, clone, modify, store; without this lock two concurrent
    // writers (e.g. a socket.io connect and the dispatcher) lose one update ---
    write_lock: Arc<Mutex<()>>,
}

impl<T: Clone> Swappable<T> {
    pub fn read(&self) -> arc_swap::Guard<Arc<T>> {
        self.value.load()
    }

    pub fn update<F>(&mut self, modify: F)
    where
        F: FnOnce(&mut T),
    {
        let _guard = self
            .write_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        let mut value = (*self.value.load_full()).clone();
        modify(&mut value);
        self.value.store(Arc::new(value));
    }
}
