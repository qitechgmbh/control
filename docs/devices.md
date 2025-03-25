# Devices

We need to implement devices to communicate, configure, and identify them.

## Implementation Checklist

Here is a checklist of what we need to implement:

- Device Port
  - derive `#[derive(Debug, Clone, Copy)]`
- Device Struct `struct EL0000`
  - derive `#[derive(Device)]` derives `Device` trait
  - derive or implement `Debug` trait
  - field `output_ts` Timestamp for the output data (only if the device has outputs)
  - field `input_ts` Timestamp for the input data (only if the device has inputs)
  - field `txpdo` instance of `EL0000TxPdo` (only if the device has inputs)
  - field `rxpdo` instance of `EL0000RxPdo` (only if the device has outputs)
  - field `configuration` instance of `EL0000Configuration` (only if configurable)
  - implement `NewDevice` trait
- TxPdo / RxPdo Struct `struct EL0000TxPdo` and/or `struct EL0000RxPdo`
  - derive `#[derive(Debug, Clone)]`
  - derive `#[derive(TxPdo)]` and/or `#[derive(RxPdo)]`
  - fields
      - tag with `#[pdo_object_index(0x1...)]` for the PDO object index
      - wrap PDO object in `Option`
- PDO Objects `struct EL0000PdoObject` or other PDO object structs
  - derive `#[derive(Debug, Clone, Default, PartialEq)]`
  - derive `#[derive(PdoObject)]`
  - tag with `#[pdo_object(bits = 16)]` for the size of the object
  - implement `TxPdoObject` and/or `RxPdoObject` traits
- Predefined PDO Assignment `enum EL0000PdoPreset` (if supported)
  - derive `#[derive(Debug, Clone)]`
  - implement `PredefinedPdoAssignment<EL0000TxPdo, EL0000RxPdo>`
- Configuration `struct EL0000Configuration` (if supported)
  - derive `#[derive(Debug, Clone)]`
  - implement `Configuration` trait
  - field `pdo_assignment` of type `EL0000PdoPreset`
  - fields for the configuration parameters

These are helpful resources:
- Other implementations of similar devices
- Configuration Values: the official datasheet
- Configuration Implementation: [Docs](./coe.md)
- PDO Schema: The "Process Data" tab in the TwinCAT Software by Beckhoff
- PDO Implementation: [Docs](./pdo.md)
- Identification: Attach the device and read the identity values with QiTech Control in the "Setup > EtherCAT > Devices" tab

ESI Files are not needed but could be an alternative reference, though not a very readable one.

## Identity

We need to extract the possible EtherCAT identity values matching this driver implementation and provide them as constants.

Beckhoff has the official vendor ID of `0x2`. The product ID is a u32 which is unique for each device like `EL0000-....`, `EL2003-....`, `EL3001-....`, etc.

Hardware revisions or variants (like `EL2521-0024`) change the revision ID.

There can be multiple revisions for one product ID which should be supported by the driver so we annotate different revision constants with a suffix like `_A`, `_B`, `_0024_A`, etc.

For each revision, we need a tuple combining all three values. This makes pattern matching easier.
```rust
pub const EL2008_VENDOR_ID: u32 = 0x2;
pub const EL2008_PRODUCT_ID: u32 = 0x07d83052;
pub const EL2008_REVISION_A: u32 = 0x00110000;

pub const EL2008_IDENTITY_A: SubDeviceIdentityTuple =
        (EL2008_VENDOR_ID, EL2008_PRODUCT_ID, EL2008_REVISION_A);
```

## Link constructor in `server` crate

Add the device constructor to the `device_from_subdevice` function in the `server` crate.
This function is used to create the device at runtime.

```rust
pub fn device_from_subdevice(
    subdevice_name: &str,
) -> Result<Arc<RwLock<dyn Device>>, anyhow::Error> {
    match subdevice_name {
        "EL0000" => Ok(Arc::new(RwLock::new(EL0000::new()))),
        // ...
    }
}
```

TODO: Refactor code to use identity tuple instead of subdevice name.
