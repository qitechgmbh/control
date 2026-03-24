use lazy_static::lazy_static;

use crate::{
    SerialDevice, SerialDeviceIdentification, machine_identification::DeviceIdentification,
    serial::devices::laser::Laser,
};

#[cfg(feature = "mock-machine")]
use crate::serial::devices::mock::MockSerialDevice;

use crate::SerialDeviceNewParams;
use anyhow::{Error, Result};
use smol::lock::RwLock;
use std::{any::TypeId, collections::HashMap, sync::Arc};

pub type SerialDeviceNewClosure = Arc<
    dyn Fn(
            &SerialDeviceNewParams,
        ) -> Result<(DeviceIdentification, Arc<RwLock<dyn SerialDevice>>), Error>
        + Send
        + Sync,
>;

#[derive(Clone)]
pub struct SerialDeviceRegistry {
    pub type_map: HashMap<TypeId, (SerialDeviceIdentification, SerialDeviceNewClosure)>,
}

impl Default for SerialDeviceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SerialDeviceRegistry {
    pub fn new() -> Self {
        Self {
            type_map: HashMap::new(),
        }
    }

    pub fn register<T: SerialDevice + 'static>(
        &mut self,
        serial_device_identification: SerialDeviceIdentification,
    ) {
        self.type_map.insert(
            TypeId::of::<T>(),
            (
                serial_device_identification,
                Arc::new(move |params| {
                    let (identification, device) = T::new_serial(params)?;
                    Ok((identification, device))
                }),
            ),
        );
    }

    pub fn new_serial_device(
        &self,
        serial_device_new_params: &SerialDeviceNewParams,
        serial_device_identification: &SerialDeviceIdentification,
    ) -> Result<(DeviceIdentification, Arc<RwLock<dyn SerialDevice>>), anyhow::Error> {
        // find serial new function by comparing ProdutConfig
        let (_, serial_new_fn) = self
            .type_map
            .values()
            .find(|(sdi, _)| sdi == serial_device_identification)
            .ok_or(anyhow::anyhow!(
                "[{}::MachineConstructor::new_machine] Machine not found",
                module_path!()
            ))?;

        // call machine new function by reference
        (serial_new_fn)(serial_device_new_params)
    }

    pub async fn downcast_arc_rwlock<T: SerialDevice + 'static>(
        &self,
        serial_device: Arc<RwLock<dyn SerialDevice>>,
    ) -> Result<Arc<RwLock<T>>, Error> {
        // Use the Any trait for type checking
        let type_id = {
            let type_id_fn = Arc::new(|device: &dyn SerialDevice| device.type_id());
            let guard = serial_device.read().await;
            let id = type_id_fn(&*guard);
            drop(guard);
            id
        };

        if TypeId::of::<T>() == type_id {
            // transmute Arc
            let arc = unsafe { Arc::from_raw(Arc::into_raw(serial_device) as *const RwLock<T>) };
            Ok(arc)
        } else {
            Err(anyhow::anyhow!(
                "[{}::MachineConstructor::downcast] Machine is not of type {}",
                module_path!(),
                std::any::type_name::<T>()
            ))
        }
    }
}

lazy_static! {
    pub static ref SERIAL_DEVICE_REGISTRY: SerialDeviceRegistry = {
        let mut sdr = SerialDeviceRegistry::new();
        sdr.register::<Laser>(SerialDeviceIdentification {
            vendor_id: 0x0403,
            product_id: 0x6001,
        });

        // Register MockSerialDevice when mock-machine feature is enabled
        #[cfg(feature = "mock-machine")]
        sdr.register::<MockSerialDevice>(SerialDeviceIdentification {
            vendor_id: 0x0001, // VENDOR_QITECH
            product_id: 0x0007, // MACHINE_MOCK
        });

        sdr
    };
}
