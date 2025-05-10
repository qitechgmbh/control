use std::time::Duration;

use super::Dre;
use control_core::{
    machines::api::MachineApi,
    socketio::{
        event::{Event, GenericEvent},
        namespace::{
            cache_duration, cache_one_event, CacheFn, CacheableEvents, Namespace,
            NamespaceCacheingLogic, NamespaceInterface,
        },
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::super::super::serial::register::SERIAL_DETECTION;

#[derive(Serialize, Debug, Clone)]
pub struct AllConnectedDevice {
    path: String,
    vendor_id: u16,
    product_id: u16,
}

#[derive(Serialize, Debug, Clone)]
pub struct AllConnectedDevicesEvent {
    devices: Vec<AllConnectedDevice>,
}
impl AllConnectedDevicesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("AllConnectedDevicesEvent", self.clone())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiameterRequestEvent {
    pub device: String,
}
impl DiameterRequestEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("DiameterRequestEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct DiameterResponseEvent {
    pub device: String,
    pub diameter: Option<f32>,
    pub error: Option<String>,
}
impl DiameterResponseEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("DiameterResponseEvent", self.clone())
    }
}

pub enum DreEvents {
    AllConnectedDevicesEvent(Event<AllConnectedDevicesEvent>),
    DiameterResponseEvent(Event<DiameterResponseEvent>),
    DiameterRequestEvent(Event<DiameterRequestEvent>)
}


#[derive(Debug)]
pub struct DreNamespace(Namespace);

impl DreNamespace {
    pub fn new() -> Self {
        Self(Namespace::new())
    }
}



impl CacheableEvents<DreEvents> for DreEvents {
    fn event_value(&self) -> Result<GenericEvent, serde_json::Error> {
        match self {
            DreEvents::AllConnectedDevicesEvent(event) => event.try_into(),
            DreEvents::DiameterResponseEvent(event) => event.try_into(),
            DreEvents::DiameterRequestEvent(event) => event.try_into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_one_hour = cache_duration(Duration::from_secs(60 * 60));
        let cache_ten_secs = cache_duration(Duration::from_secs(10));
        let cache_one = cache_one_event();

        match self {
            DreEvents::AllConnectedDevicesEvent(_) => cache_one_hour,
            DreEvents::DiameterResponseEvent(_) => cache_ten_secs,
            DreEvents::DiameterRequestEvent(_) => cache_one,
        }
    }
}


impl NamespaceCacheingLogic<DreEvents> for DreNamespace {
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
