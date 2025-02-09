use crate::types::EthercrabSubDevice;

pub trait Configuration {
    #[allow(async_fn_in_trait)]
    async fn write_config<'a>(
        &self,
        device: &'a EthercrabSubDevice<'a>,
    ) -> Result<(), anyhow::Error>;
}

pub const TX_PDO_ASSIGNMENT_REG: u16 = 0x1C13;

pub const RX_PDO_ASSIGNMENT_REG: u16 = 0x1C12;
