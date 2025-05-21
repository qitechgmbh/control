use ethercrab::{SubDevice, SubDeviceGroup, SubDevicePdi, SubDeviceRef, subdevice_group::Op};
pub type EthercrabSubDevicePreoperational<'maindevice> =
    SubDeviceRef<'maindevice, &'maindevice SubDevice>;
pub type EthercrabSubDeviceOperational<'maindevice, const MAX_PDI: usize> =
    SubDeviceRef<'maindevice, SubDevicePdi<'maindevice, MAX_PDI>>;

pub type EthercrabSubDeviceGroupOperational<const MAX_SUBDEVICES: usize, const MAX_PDI: usize> =
    SubDeviceGroup<MAX_SUBDEVICES, MAX_PDI, Op>;

pub type EthercrabSubDeviceGroupPreoperational<const MAX_SUBDEVICES: usize, const MAX_PDI: usize> =
    SubDeviceGroup<MAX_SUBDEVICES, MAX_PDI>;
