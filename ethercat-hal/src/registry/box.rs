use crate::{
    devices::{EthercatDevice, SubDeviceIdentityTuple},
    registry::{DeviceWrapper, EthercatDeviceRegistrar, EthercatDeviceRegistry},
};

/// Implementation for Box wrapper
impl<T: EthercatDevice> DeviceWrapper<T> for Box<dyn EthercatDevice> {
    fn wrap(device: T) -> Self {
        Box::new(device)
    }
}

pub type BoxedDeviceRegistry = EthercatDeviceRegistry<Box<dyn EthercatDevice>>;

/// Implement the registrar trait for BoxedDeviceRegistry
impl EthercatDeviceRegistrar for BoxedDeviceRegistry {
    fn register<T>(&mut self, device_identity: SubDeviceIdentityTuple)
    where
        T: EthercatDevice + 'static,
    {
        self.register::<T>(device_identity);
    }
}

/// Create a default BoxedDeviceRegistry with all known EtherCAT devices
pub fn create_default_registry() -> BoxedDeviceRegistry {
    let mut registry = BoxedDeviceRegistry::new();
    register_default_devices(&mut registry);
    registry
}

/// Populate a BoxedDeviceRegistry with all known EtherCAT devices
pub fn register_default_devices(registry: &mut BoxedDeviceRegistry) {
    crate::devices::register_default_devices(registry);
}
