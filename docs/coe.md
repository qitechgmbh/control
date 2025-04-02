# Configuration over EtherCAT (CoE)
Each EtherCAT has a key-value store that is accessible via SDA register reads and writes.

A value can be a primitive type like `bool`, `u8`, and so on, or an array of values.
Not all indices are writable; some are read-only.

Many indices have the same meaning across different EtherCAT devices, but some are specific to the device. Documentation about the indices can be found in the datasheet.

Important indices we need to write are:
- `0x1C12` RxPdo assignment
- `0x1C13` TxPdo assignment
- `0x80..` Device-specific configuration.

Other notable indices are:
- `0x1A..` TxPdo objects (read-only, referenced in the TxPdo assignment)
- `0x16..` RxPdo objects (read-only, referenced in the RxPdo assignment)

The configuration is not persistent and will be lost if the device is powered off. We need to write it every time we start the application. This is very beneficial because we know that it's always using the default configuration unless we explicitly change it.

## How to Create a New Device Configuration
### Config Values
Take a look into the datasheet of a device and find a table of the `0x80..` indices.
Take a look at each value and implement the whole configuration. Copy the description of each value as well as the default value to the documentation. Also, implement the `Default` trait for the configuration struct using the default values from the datasheet.

```rust
/// 0x8000 CoE
#[derive(Debug, Clone)]
pub struct EL2521Configuration {
    /// # 0x8010:02
    /// - `true` = If the watchdog timer responds, the terminal ramps with the time constant set in object 8001:08
    /// - `false` = The function is deactivated
    ///
    /// default: `false`
    pub emergency_ramp_active: bool,

    /// # 0x8010:03
    /// - `true` = The watchdog timer is deactivated
    ///
    /// The watchdog timer is activated in the delivery state.
    /// Either the manufacturer's or the user's switch-on value
    /// is output if the watchdog overflows
    ///
    /// default: `false`
    pub watchdog_timer_deactive: bool,

    // ...
}
```

Sometimes a device has multiple channels/pins/ports that all have the same configuration values. Abstract here. 

For example, the EL2522 is the 2-channel variant of the EL2521.
```rust
#[derive(Debug, Clone)]
pub struct EL2522ChannelConfiguration {
    // PTO Settings
    /// # 0x8000:01 (Ch.1) / 0x8010:01 (Ch.2)
    /// If the counter value is set to "0", the C-track goes into the "high" state
    /// Default: false (0x00)
    pub adapt_a_b_on_position_set: bool,

    // ...
}

impl Default for EL2522ChannelConfiguration {
    fn default() -> Self {
        Self {
            adapt_a_b_on_position_set: false,
            // ...
        }
    }
}

#[derive(Debug, Clone)]
pub struct EL2522Configuration {
    // ...
    pub channel1_configuration: EL2522ChannelConfiguration,
    pub channel2_configuration: EL2522ChannelConfiguration,
}

impl Default for EL2522Configuration {
    fn default() -> Self {
        Self {
            // ...
            channel1_configuration: EL2522ChannelConfiguration::default(),
            channel2_configuration: EL2522ChannelConfiguration::default(),
        }
    }
}
```

Now we need to implement the `Configuration` trait for the `EL2522Configuration` struct.
If you choose a wrong data type for a variable, this function fails.
```rust
impl Configuration for EL2521Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        device
            .sdo_write(0x8010, 0x02, self.emergency_ramp_active)
            .await?;
        device
            .sdo_write(0x8010, 0x03, self.watchdog_timer_deactive)
            .await?;
        // ...
        Ok(())
    }
}
```
In the case of multiple channels, you can implement a custom `write_config` function that takes a base index (or multiple) like `0x8010`, `0x8020`, etc. because the indices are offset by some value for each channel.
```rust
impl Configuration for EL2522Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        // Write configuration for Channel 1
        self.write_channel_config(device, 0x8000, 0x8020, &self.channel1_configuration)
            .await?;

        // Write configuration for Channel 2
        self.write_channel_config(device, 0x8010, 0x8030, &self.channel2_configuration)
            .await?;

        // ...

        Ok(())
    }
}
```

### PDO assignment

Since the PDO assignment is also part of the configuration, we need to include it in the configuration struct.
```rust
/// 0x8000 CoE
#[derive(Debug, Clone)]
pub struct EL2521Configuration {
    
    // ...

    pub pdo_assignment: EL2521PredefinedPdoAssignment,
}
```

The PDO assignment is an array of values (pointers) we defined with `#[pdo_object_index(0x1A..)]` on the PDO object field in the TxPdo or RxPdo structs. The `TxPdo` and `RxPdo` derive macros derive the `Configuration` trait for the `EL2521TxPdo` and `EL2521RxPdo` structs so we can call `write_config` on them.
```rust
impl Configuration for EL2521Configuration {
    async fn write_config<'a>(
        &self,
        device: &EthercrabSubDevicePreoperational<'a>,
    ) -> Result<(), anyhow::Error> {
        // ...

        self.pdo_assignment
            .txpdo_assignment()
            .write_config(device)
            .await?;
        self.pdo_assignment
            .rxpdo_assignment()
            .write_config(device)
            .await?;

        Ok(())
    }
}
```

### Setting the Device Configuration
The device should know how it's configured, but we cannot pass the configuration via a constructor because we first create the device and then decide on how we configure it based on the machine it belongs to. So there is a trait `ConfigurableDevice` which exposes functions to get and set the configuration.
The configuration can be saved in the device struct as well as the TxPdo and RxPdo struct from the `pdo_assignment` field.
```rust
impl ConfigurableDevice<EL2521Configuration> for EL2521 {
    async fn write_config<'maindevice>(
        &mut self,
        device: &EthercrabSubDevicePreoperational<'maindevice>,
        config: &EL2521Configuration,
    ) -> Result<(), anyhow::Error> {
        config.write_config(device).await?;
        self.configuration = config.clone();
        self.txpdo = config.pdo_assignment.txpdo_assignment();
        self.rxpdo = config.pdo_assignment.rxpdo_assignment();
        Ok(())
    }

    fn get_config(&self) -> EL2521Configuration {
        self.configuration.clone()
    }
}
```
### Putting it all together

We can now apply the configuration to an instance of the `EL2521` device. We only have to specify non-default values because we can use `..Default::default()` to fill in the rest. Here the default predefined PDO assignment is used. The whole configuration is simply written to the device. 

```rust
let el2521 = EL2521::new();
el2521
    .write()
    .await
    .write_config(
        &subdevice,
        &EL2521Configuration {
            direct_input_mode: true,
            ..Default::default()
        },
    )
    .await?;
```
