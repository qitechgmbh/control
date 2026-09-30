use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

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
<<<<<<< HEAD
    // pub machines: Swappable<HashMap<MachineInstanceIdentification, MachineInstance>>,
=======
    pub machines: Swappable<HashMap<MachineInstanceIdentification, MachineInstance>>,
>>>>>>> 8e49141 (Jse control v2 (#1680))
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
<<<<<<< HEAD

    #[allow(unused)]
=======
>>>>>>> 8e49141 (Jse control v2 (#1680))
    pub records: Vec<ConfigPropertyEventRecord>,
}

#[derive(Clone)]
pub struct StatePropertyInfo {
    pub value: ScalarValue,
<<<<<<< HEAD

    #[allow(unused)]
=======
>>>>>>> 8e49141 (Jse control v2 (#1680))
    pub records: Vec<StatePropertyEventRecord>,
}

#[derive(Clone)]
pub struct MeasurementInfo {
    pub value: Option<f64>,
}

#[derive(Default, Clone)]
pub struct Swappable<T: Clone>(Arc<ArcSwap<T>>);

impl<T: Clone> Swappable<T> {
    pub fn read(&self) -> arc_swap::Guard<Arc<T>> {
        self.0.load()
    }

    pub fn update<F>(&mut self, modify: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = (*self.0.load_full()).clone();
        modify(&mut value);
        self.0.store(Arc::new(value));
    }
}
