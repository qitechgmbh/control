use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup, SubDevicePdi, SubDeviceRef};

use super::config::{MAX_SUBDEVICES, PDI_LEN};

pub async fn find_device<'maindevice, 'group>(
    devices: &'group SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    maindevice: &'maindevice MainDevice<'group>,
    address: u16,
) -> Option<SubDeviceRef<'group, SubDevicePdi<'group, PDI_LEN>>>
where
    'maindevice: 'group,
{
    let device = devices
        .iter(maindevice)
        .find(|device| device.configured_address() == address);

    device
}
