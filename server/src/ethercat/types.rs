use ethercrab::{SubDevice, SubDeviceRef};

pub type EthercrabSubDevice<'maindevice> = SubDeviceRef<'maindevice, &'maindevice SubDevice>;
