use qitech_framework::MachineSchema;
use qitech_framework::RuntimeInitEvent;
use qitech_framework::RuntimeReport;
use qitech_framework_hub::Listener;

use crate::api::LegacySharedState;
use crate::api::legacy;
use crate::api::types::SharedState;

pub struct SocketIODispatcher {
    pub state: SharedState,
    pub state_legacy: LegacySharedState,
}

impl SocketIODispatcher {
    pub fn new(state: SharedState, state_legacy: LegacySharedState) -> Self {
        Self {
            state,
            state_legacy,
        }
    }
}

impl Listener for SocketIODispatcher {
    fn on_schema_sync(&mut self, schema: &MachineSchema) {
        self.state
            .schemas
            .update(|schemas| _ = schemas.insert(schema.identification, schema.clone()));
    }

    fn on_init_event_received(&mut self, event: &RuntimeInitEvent) {
        match event.clone() {
            RuntimeInitEvent::EtherCATStateUpdate(state) => {
                self.state_legacy
                    .ns_main
                    .update(|ns| ns.set_ecat_state(state));
            }

            RuntimeInitEvent::EtherCATDeviceInitializationCompleted { devices } => {
                let mut devices_transformed = Vec::new();

                for dev in devices {
                    devices_transformed.push(legacy::EtherCATDeviceMetadata::from(dev));
                }

                self.state_legacy
                    .ns_main
                    .update(|ns| ns.set_ecat_devices(devices_transformed));
            }

            RuntimeInitEvent::MachineBuildCompleted { ident, result } => {
                let schemas = self.state.schemas.read();

                let Some(schema) = schemas.get(&ident.machine) else {
                    return;
                };

                self.state_legacy
                    .ns_main
                    .update(|ns| ns.add_machine(ident, result.map_err(|e| e.to_string())));

                self.state_legacy
                    .ns_machines
                    .update(|ns| ns.register(ident, schema));
            }

            _ => {}
        }
    }

    fn on_report_received(&mut self, report: &RuntimeReport) {
        self.state_legacy
            .ns_machines
            .update(|ns| ns.update(&report.machines));
    }
}
