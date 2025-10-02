use std::future::Future;

use crate::helpers::ethercrab_types::EthercrabSubDevicePreoperational;

pub trait Configuration {
    #[allow(async_fn_in_trait)]
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error>;
}

/// Wraps functionality of [`Configuration`] and adds getter/setter for the config
pub trait ConfigurableDevice<C>
where
    C: Configuration + Clone,
{
    /// Write the config to the subdevice & saves it in the device
    ///
    /// The implementation should call [`Configuration::write_config`] to write the config to the subdevice
    /// It can only be called in preoperational state
    ///
    /// Then the implementation should save the config in the device and also the `txpdo` and `rxpdo`
    ///
    /// Example:
    /// ```ignore
    /// use crate::coe::{ConfigurableDevice, Configuration};
    /// use crate::devices::el3001::{EL3001, EL3001Configuration};
    ///
    /// impl ConfigurableDevice<EL3001Configuration> for EL3001 {
    ///     async fn write_config<'maindevice>(
    ///         &mut self,
    ///         device: &EthercrabSubDevicePreoperational<'maindevice>,
    ///         config: &EL3001Configuration,
    ///     ) -> Result<(), anyhow::Error> {
    ///         config.write_config(device).await?;
    ///         self.configuration = config.clone();
    ///         self.txpdo = config.pdo_assignment.txpdo_assignment();
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn write_config(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'_>,
        config: &C,
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Returns the current config of the device
    ///
    /// Example:
    /// ```ignore
    /// use crate::coe::{ConfigurableDevice, Configuration};
    /// use crate::devices::el3001::{EL3001, EL3001Configuration};
    ///
    /// impl ConfigurableDevice<EL3001Configuration> for EL3001 {
    ///    fn get_config(&self) -> EL3001Configuration {
    ///        self.configuration.clone()
    ///    }
    /// }
    ///
    fn get_config(&self) -> C;
}

pub const TX_PDO_ASSIGNMENT_REG: u16 = 0x1C13;

pub const RX_PDO_ASSIGNMENT_REG: u16 = 0x1C12;
