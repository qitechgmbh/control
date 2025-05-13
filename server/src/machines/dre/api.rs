use std::time::Duration;

use super::DreMachine;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, NamespaceInterface,
            cache_duration,
        },
    },
};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Debug, Clone)]
pub struct AllConnectedDevice {
    path: String,
    vendor_id: u16,
    product_id: u16,
}

#[derive(Serialize, Debug, Clone)]
pub struct DiameterEvent {
    pub diameter: Option<f32>,
}

impl DiameterEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("DiameterResponseEvent", self.clone())
    }
}

pub enum DreEvents {
    DiameterEvent(Event<DiameterEvent>),
}

#[derive(Debug)]
pub struct DreMachineNamespace(Namespace);

impl DreMachineNamespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}

impl CacheableEvents<DreEvents> for DreEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            DreEvents::DiameterEvent(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_ten_secs = cache_duration(Duration::from_secs(10));

        match self {
            DreEvents::DiameterEvent(_) => cache_ten_secs,
        }
    }
}

impl NamespaceCacheingLogic<DreEvents> for DreMachineNamespace {
    fn emit_cached(&mut self, events: DreEvents) {
        let event = match events.event_value() {
            Ok(event) => event,
            Err(err) => {
                log::error!(
                    "[{}::emit_cached] Failed to event.event_value(): {:?}",
                    module_path!(),
                    err
                );
                return;
            }
        };
        let buffer_fn = events.event_cache_fn();
        self.0.emit_cached(&event, buffer_fn);
    }
}

// impl DreNamespace {
//     // This function handles events and updates the connected devices or responds to requests
//     pub fn process_event(&mut self, event: DreEvents) {
//         match event {
//             DreEvents::AllConnectedDevicesEvent(_) => {
//                 // Send the updated list of connected devices
//                 let connected_devices = self.get_all_connected_devices();
//                 let event = AllConnectedDevicesEvent {
//                     devices: connected_devices,
//                 };
//                 self.emit_cached(DreEvents::AllConnectedDevicesEvent(event.build()));
//             }
//             DreEvents::DiameterRequestEvent(request_event) => {
//                 // Respond to the diameter request
//                 let response = self.handle_diameter_request(request_event.data);
//                 self.emit_cached(DreEvents::DiameterResponseEvent(response.build()));
//             }
//             _ => {
//                 // Optionally handle other events if needed
//             }
//         }
//     }

//     // Function to get the list of all connected devices
//     async fn get_all_connected_devices(&self) -> Vec<AllConnectedDevice> {
//         // Assuming SERIAL_DETECTION gives you a list of connected devices
//         let mut list = Vec::new();
//         let check_list = SERIAL_DETECTION.read().await;
//         for element in check_list.ports.clone(){
//             list.push(AllConnectedDevice{
//                 path: element.0,
//                 vendor_id: element.1.vid,
//                 product_id: element.1.pid,
//             });
//         };
//         list
//     }

//     async fn handle_diameter_request(&self, request_event: DiameterRequestEvent) -> DiameterResponseEvent {
//         let device_name = request_event.device;
//         let check_list = SERIAL_DETECTION.read().await;

//         let diameter_value = check_list.connected_serial_usb.get(&device_name);
//         let error_message;
//         let value:Option<f32>;
//         match diameter_value{
//             Some(diam) =>{
//                 match diam{
//                     Ok(d) => {
//                         let x = d.read().await;
//                         value = Some();
//                         error_message = None},
//                     Err(e) => {
//                         error_message = Some(e);
//                         value =None}
//                 }
//             },
//             None => {
//                 value = None;
//                 error_message = Some(&anyhow::anyhow!("No diameter found"))
//             }
//         };

//         DiameterResponseEvent {
//             device: device_name,
//             diameter: value,
//             error: error_message,
//         }
//     }
// }

impl MachineApi for DreMachine {
    fn api_mutate(&mut self, _request_body: Value) -> Result<(), anyhow::Error> {
        // let mutation: Mutation = serde_json::from_value(request_body)?;
        // match mutation {}
        Ok(())
    }

    fn api_event_namespace(&mut self) -> &mut dyn NamespaceInterface {
        &mut self.namespace.0
    }
}
