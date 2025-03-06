use ethercrab::{
    EtherCrabWireReadSized, MainDevice, SubDevice, SubDeviceIdentity, SubDevicePdi, SubDeviceRef,
};
pub type EthercrabSubDevicePreoperational<'maindevice> =
    SubDeviceRef<'maindevice, &'maindevice SubDevice>;
pub type EthercrabSubDeviceOperational<'maindevice, const PDI_LEN: usize> =
    SubDeviceRef<'maindevice, SubDevicePdi<'maindevice, PDI_LEN>>;
pub enum EthercrabSubDevice<'maindevice, 'subdevice, const PDI_LEN: usize> {
    Preoperational(&'subdevice EthercrabSubDevicePreoperational<'maindevice>),
    Operational(&'subdevice EthercrabSubDeviceOperational<'maindevice, PDI_LEN>),
}

impl EthercrabSubDevice<'_, '_, 64> {
    pub fn identity(&self) -> SubDeviceIdentity {
        match self {
            EthercrabSubDevice::Preoperational(subdevice) => subdevice.identity(),
            EthercrabSubDevice::Operational(subdevice) => subdevice.identity(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            EthercrabSubDevice::Preoperational(subdevice) => subdevice.name(),
            EthercrabSubDevice::Operational(subdevice) => subdevice.name(),
        }
    }

    pub async fn eeprom_read<T>(
        &self,
        maindevice: &MainDevice<'_>,
        start_word: u16,
    ) -> Result<T, ethercrab::error::Error>
    where
        T: EtherCrabWireReadSized,
    {
        match self {
            EthercrabSubDevice::Preoperational(subdevice) => {
                subdevice.eeprom_read(maindevice, start_word).await
            }
            EthercrabSubDevice::Operational(subdevice) => {
                subdevice.eeprom_read(maindevice, start_word).await
            }
        }
    }
}
