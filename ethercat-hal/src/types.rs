use atomic_refcell::AtomicRefMut;
use ethercrab::{subdevice_group::Op, SubDeviceGroup, SubDevicePdi, SubDeviceRef};

pub type EthercrabSubDevicePreoperational<'maindevice> =
    SubDeviceRef<'maindevice, AtomicRefMut<'maindevice, ethercrab::SubDevice>>;

pub type EthercrabSubDeviceOperational<'maindevice> =
    SubDeviceRef<'maindevice, SubDevicePdi<'maindevice>>;

pub type EthercrabSubDeviceGroupOperational<const MAX_SUBDEVICES: usize, const MAX_PDI: usize> =
    SubDeviceGroup<MAX_SUBDEVICES, MAX_PDI, Op>;

pub type EthercrabSubDeviceGroupPreoperational<const MAX_SUBDEVICES: usize, const MAX_PDI: usize> =
    SubDeviceGroup<MAX_SUBDEVICES, MAX_PDI>;
