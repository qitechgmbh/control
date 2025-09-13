# Crate Documentation

**Version:** 0.1.0

**Format Version:** 43

# Module `server`

## Modules

## Module `app_state`

```rust
pub mod app_state { /* ... */ }
```

### Types

#### Struct `SocketioSetup`

```rust
pub struct SocketioSetup {
    pub socketio: smol::lock::RwLock<Option<socketioxide::SocketIo>>,
    pub namespaces: smol::lock::RwLock<crate::socketio::namespaces::Namespaces>,
    pub socket_queue_tx: smol::channel::Sender<(socketioxide::extract::SocketRef, std::sync::Arc<control_core::socketio::event::GenericEvent>)>,
    pub socket_queue_rx: smol::channel::Receiver<(socketioxide::extract::SocketRef, std::sync::Arc<control_core::socketio::event::GenericEvent>)>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `socketio` | `smol::lock::RwLock<Option<socketioxide::SocketIo>>` |  |
| `namespaces` | `smol::lock::RwLock<crate::socketio::namespaces::Namespaces>` |  |
| `socket_queue_tx` | `smol::channel::Sender<(socketioxide::extract::SocketRef, std::sync::Arc<control_core::socketio::event::GenericEvent>)>` |  |
| `socket_queue_rx` | `smol::channel::Receiver<(socketioxide::extract::SocketRef, std::sync::Arc<control_core::socketio::event::GenericEvent>)>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `SerialSetup`

```rust
pub struct SerialSetup {
    pub serial_detection: control_core::serial::serial_detection::SerialDetection<''static>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `serial_detection` | `control_core::serial::serial_detection::SerialDetection<''static>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `AppState`

```rust
pub struct AppState {
    pub socketio_setup: SocketioSetup,
    pub ethercat_setup: std::sync::Arc<smol::lock::RwLock<Option<EthercatSetup>>>,
    pub serial_setup: std::sync::Arc<smol::lock::RwLock<SerialSetup>>,
    pub machines: std::sync::Arc<smol::lock::RwLock<control_core::machines::manager::MachineManager>>,
    pub performance_metrics: std::sync::Arc<smol::lock::RwLock<crate::performance_metrics::EthercatPerformanceMetrics>>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `socketio_setup` | `SocketioSetup` |  |
| `ethercat_setup` | `std::sync::Arc<smol::lock::RwLock<Option<EthercatSetup>>>` |  |
| `serial_setup` | `std::sync::Arc<smol::lock::RwLock<SerialSetup>>` |  |
| `machines` | `std::sync::Arc<smol::lock::RwLock<control_core::machines::manager::MachineManager>>` |  |
| `performance_metrics` | `std::sync::Arc<smol::lock::RwLock<crate::performance_metrics::EthercatPerformanceMetrics>>` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Type Alias `Machines`

```rust
pub type Machines = std::collections::HashMap<control_core::machines::identification::MachineIdentificationUnique, Result<std::sync::Arc<smol::lock::RwLock<dyn Machine>>, anyhow::Error>>;
```

#### Struct `EthercatSetup`

```rust
pub struct EthercatSetup {
    pub devices: Vec<(control_core::machines::identification::DeviceIdentification, std::sync::Arc<smol::lock::RwLock<dyn EthercatDevice>>)>,
    pub group: ethercrab::SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, ethercrab::subdevice_group::Op>,
    pub maindevice: ethercrab::MainDevice<''static>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `devices` | `Vec<(control_core::machines::identification::DeviceIdentification, std::sync::Arc<smol::lock::RwLock<dyn EthercatDevice>>)>` | All Ethercat devices<br>Device-Specific interface for all devices<br>Same length and order as SubDevices inside `group` (index = subdevice_index) |
| `group` | `ethercrab::SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, ethercrab::subdevice_group::Op>` | All Ethercat devices<br>Generic interface for all devices<br>Needed to interface with the devices on an Ethercat level |
| `maindevice` | `ethercrab::MainDevice<''static>` | The Ethercat main device<br>Needed to interface with the devices |

##### Implementations

###### Methods

- ```rust
  pub fn new(devices: Vec<(DeviceIdentification, Arc<RwLock<dyn EthercatDevice>>)>, group: SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>, maindevice: MainDevice<''static>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `ethercat`

```rust
pub mod ethercat { /* ... */ }
```

### Modules

## Module `config`

```rust
pub mod config { /* ... */ }
```

### Constants and Statics

#### Constant `MAX_SUBDEVICES`

Maximum number of SubDevices that can be stored. This must be a power of 2 greater than 1.

```rust
pub const MAX_SUBDEVICES: usize = 16;
```

#### Constant `MAX_PDU_DATA`

Maximum PDU data payload size - set this to the max PDI size or higher.

```rust
pub const MAX_PDU_DATA: usize = _;
```

#### Constant `MAX_FRAMES`

Maximum number of EtherCAT frames that can be in flight at any one time.

```rust
pub const MAX_FRAMES: usize = 16;
```

#### Constant `PDI_LEN`

Maximum total PDI length.

```rust
pub const PDI_LEN: usize = 256;
```

## Module `init`

```rust
pub mod init { /* ... */ }
```

### Functions

#### Function `init_ethercat`

```rust
pub fn init_ethercat(thread_panic_tx: smol::channel::Sender<crate::panic::PanicDetails>, app_state: std::sync::Arc<crate::app_state::AppState>) -> Result<(), anyhow::Error> { /* ... */ }
```

## Module `setup`

```rust
pub mod setup { /* ... */ }
```

### Functions

#### Function `setup_loop`

```rust
pub async fn setup_loop(thread_panic_tx: smol::channel::Sender<crate::panic::PanicDetails>, interface: &str, app_state: std::sync::Arc<crate::app_state::AppState>) -> Result<(), anyhow::Error> { /* ... */ }
```

## Module `logging`

```rust
pub mod logging { /* ... */ }
```

### Modules

## Module `fmt`

**Attributes:**

- `#[<cfg>(feature = "tracing-fmt")]`

```rust
pub mod fmt { /* ... */ }
```

### Functions

#### Function `init_fmt_tracing`

```rust
pub fn init_fmt_tracing<S>() -> Box<dyn Layer<S> + Send + Sync + ''static>
where
    S: tracing::Subscriber + for<''lookup> tracing_subscriber::registry::LookupSpan<''lookup> { /* ... */ }
```

### Functions

#### Function `init_tracing`

Initialize the basic tracing system (without OpenTelemetry if enabled)
OpenTelemetry layer is deferred until async runtime is available

```rust
pub fn init_tracing() { /* ... */ }
```

## Module `loop`

```rust
pub mod loop { /* ... */ }
```

### Functions

#### Function `init_loop`

```rust
pub fn init_loop(thread_panic_tx: smol::channel::Sender<crate::panic::PanicDetails>, app_state: std::sync::Arc<crate::app_state::AppState>) -> Result<(), anyhow::Error> { /* ... */ }
```

#### Function `loop_once`

```rust
pub async fn loop_once<''maindevice>(app_state: std::sync::Arc<crate::app_state::AppState>) -> Result<(), anyhow::Error> { /* ... */ }
```

## Module `machines`

```rust
pub mod machines { /* ... */ }
```

### Modules

## Module `buffer1`

```rust
pub mod buffer1 { /* ... */ }
```

### Modules

## Module `act`

```rust
pub mod act { /* ... */ }
```

## Module `api`

```rust
pub mod api { /* ... */ }
```

### Types

#### Struct `LiveValuesEvent`

```rust
pub struct LiveValuesEvent {
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LiveValuesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> LiveValuesEvent { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `StateEvent`

```rust
pub struct StateEvent {
    pub mode_state: ModeState,
    pub connected_machine_state: ConnectedMachineState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mode_state` | `ModeState` | mode state |
| `connected_machine_state` | `ConnectedMachineState` | connected machine state |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> StateEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `BufferV1Events`

```rust
pub enum BufferV1Events {
    LiveValues(control_core::socketio::event::Event<LiveValuesEvent>),
    State(control_core::socketio::event::Event<StateEvent>),
}
```

##### Variants

###### `LiveValues`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<LiveValuesEvent>` |  |

###### `State`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<StateEvent>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **CacheableEvents**
  - ```rust
    fn event_value(self: &Self) -> GenericEvent { /* ... */ }
    ```

  - ```rust
    fn event_cache_fn(self: &Self) -> CacheFn { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: BufferV1Events) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ModeState`

```rust
pub struct ModeState {
    pub mode: super::BufferV1Mode,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mode` | `super::BufferV1Mode` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ModeState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Mode`

```rust
pub enum Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}
```

##### Variants

###### `Standby`

###### `FillingBuffer`

###### `EmptyingBuffer`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> Mode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &Mode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ConnectedMachineState`

```rust
pub struct ConnectedMachineState {
    pub machine_identification_unique: Option<control_core::machines::identification::MachineIdentificationUnique>,
    pub is_available: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `machine_identification_unique` | `Option<control_core::machines::identification::MachineIdentificationUnique>` | Connected Machine |
| `is_available` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ConnectedMachineState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &ConnectedMachineState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Mutation`

```rust
pub(in ::machines::buffer1::api) enum Mutation {
    SetBufferMode(super::BufferV1Mode),
    SetConnectedMachine(control_core::machines::identification::MachineIdentificationUnique),
    DisconnectMachine(control_core::machines::identification::MachineIdentificationUnique),
}
```

##### Variants

###### `SetBufferMode`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `super::BufferV1Mode` |  |

###### `SetConnectedMachine`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::machines::identification::MachineIdentificationUnique` |  |

###### `DisconnectMachine`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::machines::identification::MachineIdentificationUnique` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `Buffer1Namespace`

```rust
pub struct Buffer1Namespace {
    pub namespace: control_core::socketio::namespace::Namespace,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `control_core::socketio::namespace::Namespace` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: BufferV1Events) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `buffer_tower_controller`

```rust
pub mod buffer_tower_controller { /* ... */ }
```

### Types

#### Struct `BufferTowerController`

```rust
pub struct BufferTowerController {
    pub(in ::machines::buffer1::buffer_tower_controller) enabled: bool,
    pub stepper_driver: ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `enabled` | `bool` |  |
| `stepper_driver` | `ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1` | Stepper driver. Controls buffer stepper motor |

##### Implementations

###### Methods

- ```rust
  pub fn new(driver: StepperVelocityEL70x1) -> Self { /* ... */ }
  ```

- ```rust
  pub fn set_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `new`

```rust
pub mod new { /* ... */ }
```

### Types

#### Struct `BufferV1`

```rust
pub struct BufferV1 {
    pub buffer_tower_controller: buffer_tower_controller::BufferTowerController,
    pub(in ::machines::buffer1) namespace: api::Buffer1Namespace,
    pub(in ::machines::buffer1) last_measurement_emit: std::time::Instant,
    pub machine_manager: std::sync::Weak<smol::lock::RwLock<control_core::machines::manager::MachineManager>>,
    pub machine_identification_unique: control_core::machines::identification::MachineIdentificationUnique,
    pub connected_winder: Option<control_core::machines::ConnectedMachine<std::sync::Weak<smol::lock::Mutex<crate::machines::winder2::Winder2>>>>,
    pub(in ::machines::buffer1) mode: BufferV1Mode,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `buffer_tower_controller` | `buffer_tower_controller::BufferTowerController` |  |
| `namespace` | `api::Buffer1Namespace` |  |
| `last_measurement_emit` | `std::time::Instant` |  |
| `machine_manager` | `std::sync::Weak<smol::lock::RwLock<control_core::machines::manager::MachineManager>>` |  |
| `machine_identification_unique` | `control_core::machines::identification::MachineIdentificationUnique` |  |
| `connected_winder` | `Option<control_core::machines::ConnectedMachine<std::sync::Weak<smol::lock::Mutex<crate::machines::winder2::Winder2>>>>` |  |
| `mode` | `BufferV1Mode` |  |

##### Implementations

###### Methods

- ```rust
  pub fn emit_live_values(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn emit_state(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn fill_buffer(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn empty_buffer(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn switch_to_standby(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn switch_to_filling(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn switch_to_emptying(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn switch_mode(self: &mut Self, mode: BufferV1Mode) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::buffer1) fn set_mode_state(self: &mut Self, mode: BufferV1Mode) { /* ... */ }
  ```

- ```rust
  pub fn set_connected_winder(self: &mut Self, machine_identification_unique: MachineIdentificationUnique) { /* ... */ }
  ```
  set connected winder

- ```rust
  pub fn disconnect_winder(self: &mut Self, machine_identification_unique: MachineIdentificationUnique) { /* ... */ }
  ```
  disconnect winder

- ```rust
  pub fn reverse_connect(self: &mut Self) { /* ... */ }
  ```
  initiate connection from winder to buffer

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut std::fmt::Formatter<''_>) -> std::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Machine**
  - ```rust
    fn as_any(self: &Self) -> &dyn Any { /* ... */ }
    ```

- **MachineAct**
  - ```rust
    fn act(self: &mut Self, now: Instant) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + ''_>> { /* ... */ }
    ```

- **MachineApi**
  - ```rust
    fn api_mutate(self: &mut Self, request_body: Value) -> Result<(), anyhow::Error> { /* ... */ }
    ```

  - ```rust
    fn api_event_namespace(self: &mut Self) -> &mut Namespace { /* ... */ }
    ```

- **MachineNewTrait**
  - ```rust
    fn new<''maindevice>(params: &MachineNewParams<''_, ''_, ''_, ''_, ''_, ''_, ''_>) -> Result<Self, Error> { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToString**
  - ```rust
    fn to_string(self: &Self) -> String { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `BufferV1Mode`

```rust
pub enum BufferV1Mode {
    Standby,
    FillingBuffer,
    EmptyingBuffer,
}
```

##### Variants

###### `Standby`

###### `FillingBuffer`

###### `EmptyingBuffer`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> BufferV1Mode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &BufferV1Mode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `extruder1`

```rust
pub mod extruder1 { /* ... */ }
```

### Modules

## Module `act`

```rust
pub mod act { /* ... */ }
```

## Module `api`

```rust
pub mod api { /* ... */ }
```

### Types

#### Struct `MotorStatusValues`

```rust
pub struct MotorStatusValues {
    pub(in ::machines::extruder1::api) screw_rpm: f64,
    pub(in ::machines::extruder1::api) frequency: f64,
    pub(in ::machines::extruder1::api) voltage: f64,
    pub(in ::machines::extruder1::api) current: f64,
    pub(in ::machines::extruder1::api) power: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `screw_rpm` | `f64` |  |
| `frequency` | `f64` |  |
| `voltage` | `f64` |  |
| `current` | `f64` |  |
| `power` | `f64` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MotorStatusValues { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> MotorStatusValues { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(status: MotorStatus) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LiveValuesEvent`

```rust
pub struct LiveValuesEvent {
    pub motor_status: MotorStatusValues,
    pub pressure: f64,
    pub nozzle_temperature: f64,
    pub front_temperature: f64,
    pub back_temperature: f64,
    pub middle_temperature: f64,
    pub nozzle_power: f64,
    pub front_power: f64,
    pub back_power: f64,
    pub middle_power: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `motor_status` | `MotorStatusValues` | screw rpm |
| `pressure` | `f64` | pressure in bar |
| `nozzle_temperature` | `f64` | nozzle temperature in celsius |
| `front_temperature` | `f64` | front temperature in celsius |
| `back_temperature` | `f64` | back temperature in celsius |
| `middle_temperature` | `f64` | middle temperature in celsius |
| `nozzle_power` | `f64` | nozzle heating power in watts |
| `front_power` | `f64` | front heating power in watts |
| `back_power` | `f64` | back heating power in watts |
| `middle_power` | `f64` | middle heating power in watts |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LiveValuesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> LiveValuesEvent { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `StateEvent`

```rust
pub struct StateEvent {
    pub is_default_state: bool,
    pub rotation_state: RotationState,
    pub mode_state: ModeState,
    pub regulation_state: RegulationState,
    pub pressure_state: PressureState,
    pub screw_state: ScrewState,
    pub heating_states: HeatingStates,
    pub extruder_settings_state: ExtruderSettingsState,
    pub inverter_status_state: InverterStatusState,
    pub pid_settings: PidSettingsStates,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `is_default_state` | `bool` |  |
| `rotation_state` | `RotationState` | rotation state |
| `mode_state` | `ModeState` | mode state |
| `regulation_state` | `RegulationState` | regulation state |
| `pressure_state` | `PressureState` | pressure state |
| `screw_state` | `ScrewState` | screw state |
| `heating_states` | `HeatingStates` | heating states |
| `extruder_settings_state` | `ExtruderSettingsState` | extruder settings state |
| `inverter_status_state` | `InverterStatusState` | inverter status state |
| `pid_settings` | `PidSettingsStates` | pid settings |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **BuildEvent**
  - ```rust
    fn build(self: &Self) -> control_core::socketio::event::Event<Self> { /* ... */ }
    ```
    Implemented by the BuildEvent derive macro

- **Clone**
  - ```rust
    fn clone(self: &Self) -> StateEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &StateEvent) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `RotationState`

```rust
pub struct RotationState {
    pub forward: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `forward` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> RotationState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &RotationState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ModeState`

```rust
pub struct ModeState {
    pub mode: super::ExtruderV2Mode,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mode` | `super::ExtruderV2Mode` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ModeState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &ModeState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `RegulationState`

```rust
pub struct RegulationState {
    pub uses_rpm: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `uses_rpm` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> RegulationState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &RegulationState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `PressureState`

```rust
pub struct PressureState {
    pub target_bar: f64,
    pub wiring_error: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `target_bar` | `f64` |  |
| `wiring_error` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PressureState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &PressureState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ScrewState`

```rust
pub struct ScrewState {
    pub target_rpm: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `target_rpm` | `f64` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ScrewState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &ScrewState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `HeatingStates`

```rust
pub struct HeatingStates {
    pub nozzle: HeatingState,
    pub front: HeatingState,
    pub back: HeatingState,
    pub middle: HeatingState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `nozzle` | `HeatingState` |  |
| `front` | `HeatingState` |  |
| `back` | `HeatingState` |  |
| `middle` | `HeatingState` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> HeatingStates { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &HeatingStates) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `HeatingState`

```rust
pub struct HeatingState {
    pub target_temperature: f64,
    pub wiring_error: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `target_temperature` | `f64` |  |
| `wiring_error` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> HeatingState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &HeatingState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ExtruderSettingsState`

```rust
pub struct ExtruderSettingsState {
    pub pressure_limit: f64,
    pub pressure_limit_enabled: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `pressure_limit` | `f64` |  |
| `pressure_limit_enabled` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ExtruderSettingsState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &ExtruderSettingsState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `InverterStatusState`

```rust
pub struct InverterStatusState {
    pub running: bool,
    pub forward_running: bool,
    pub reverse_running: bool,
    pub up_to_frequency: bool,
    pub overload_warning: bool,
    pub no_function: bool,
    pub output_frequency_detection: bool,
    pub abc_fault: bool,
    pub fault_occurence: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `running` | `bool` | RUN (Inverter running) |
| `forward_running` | `bool` | Forward running motor spins forward |
| `reverse_running` | `bool` | Reverse running motor spins backwards |
| `up_to_frequency` | `bool` | Up to frequency, SU not completely sure what its for |
| `overload_warning` | `bool` | overload warning OL |
| `no_function` | `bool` | No function, its described that way in the datasheet |
| `output_frequency_detection` | `bool` | FU Output Frequency Detection |
| `abc_fault` | `bool` | ABC (Fault) |
| `fault_occurence` | `bool` | is True when a fault occured |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> InverterStatusState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &InverterStatusState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `PidSettings`

```rust
pub struct PidSettings {
    pub ki: f64,
    pub kp: f64,
    pub kd: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `ki` | `f64` |  |
| `kp` | `f64` |  |
| `kd` | `f64` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PidSettings { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &PidSettings) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `PidSettingsStates`

```rust
pub struct PidSettingsStates {
    pub temperature: PidSettings,
    pub pressure: PidSettings,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `temperature` | `PidSettings` |  |
| `pressure` | `PidSettings` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PidSettingsStates { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &PidSettingsStates) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `ExtruderV2Events`

```rust
pub enum ExtruderV2Events {
    LiveValues(control_core::socketio::event::Event<LiveValuesEvent>),
    State(control_core::socketio::event::Event<StateEvent>),
}
```

##### Variants

###### `LiveValues`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<LiveValuesEvent>` |  |

###### `State`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<StateEvent>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **CacheableEvents**
  - ```rust
    fn event_value(self: &Self) -> GenericEvent { /* ... */ }
    ```

  - ```rust
    fn event_cache_fn(self: &Self) -> CacheFn { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: ExtruderV2Events) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Mutation`

```rust
pub(in ::machines::extruder1::api) enum Mutation {
    SetInverterRotationDirection(bool),
    SetInverterTargetPressure(f64),
    SetInverterTargetRpm(f64),
    SetInverterRegulation(bool),
    SetExtruderMode(super::ExtruderV2Mode),
    SetFrontHeatingTargetTemperature(f64),
    SetBackHeatingTargetTemperature(f64),
    SetMiddleHeatingTemperature(f64),
    SetNozzleHeatingTemperature(f64),
    SetExtruderPressureLimit(f64),
    SetExtruderPressureLimitIsEnabled(bool),
    SetPressurePidSettings(PidSettings),
    ResetInverter(bool),
}
```

##### Variants

###### `SetInverterRotationDirection`

INVERTER
Frequency Control

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `SetInverterTargetPressure`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetInverterTargetRpm`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetInverterRegulation`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `SetExtruderMode`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `super::ExtruderV2Mode` |  |

###### `SetFrontHeatingTargetTemperature`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetBackHeatingTargetTemperature`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetMiddleHeatingTemperature`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetNozzleHeatingTemperature`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetExtruderPressureLimit`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetExtruderPressureLimitIsEnabled`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `SetPressurePidSettings`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `PidSettings` |  |

###### `ResetInverter`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ExtruderV2Namespace`

```rust
pub struct ExtruderV2Namespace {
    pub namespace: control_core::socketio::namespace::Namespace,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `control_core::socketio::namespace::Namespace` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: ExtruderV2Events) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `mitsubishi_cs80`

```rust
pub mod mitsubishi_cs80 { /* ... */ }
```

### Types

#### Enum `MitsubishiCS80Register`

Specifies all System environment Variables
Register addresses are calculated as follows: Register-value 40002 -> address: 40002-40001 -> actual address in request:0x1

```rust
pub(in ::machines::extruder1::mitsubishi_cs80) enum MitsubishiCS80Register {
    InverterReset,
    InverterStatusAndControl,
    RunningFrequencyRAM,
    MotorStatus,
}
```

##### Variants

###### `InverterReset`

Register 40002

###### `InverterStatusAndControl`

Register 40003
Register 40004
Register 40006
Register 40007
Register 40009

###### `RunningFrequencyRAM`

Register 40010
Register 40014

###### `MotorStatus`

Register 40015
Register 40201

##### Implementations

###### Methods

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) const fn address(self: Self) -> u16 { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) const fn address_be_bytes(self: Self) -> [u8; 2] { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MitsubishiCS80Register { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Copy**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `MitsubishiCS80Requests`

These Requests Serve as Templates for controlling the inverter

```rust
pub enum MitsubishiCS80Requests {
    None,
    ResetInverter,
    ClearAllParameters,
    ClearNonCommunicationParameter,
    ClearNonCommunicationParameters,
    ReadInverterStatus,
    StopMotor,
    StartForwardRotation,
    StartReverseRotation,
    ReadRunningFrequency,
    WriteRunningFrequency,
    ReadMotorStatus,
    WriteParameter,
}
```

##### Variants

###### `None`

###### `ResetInverter`

Register 40002, Reset/Restart the Inverter

###### `ClearAllParameters`

Register 40004, Clear ALL parameters

###### `ClearNonCommunicationParameter`

Register 40006, Clear a non communication parameter

###### `ClearNonCommunicationParameters`

Register 40007, Clear all Non Communication related Parameters

###### `ReadInverterStatus`

Register 40009, Read Inverter Status

###### `StopMotor`

Register 40009, Stops the Motor

###### `StartForwardRotation`

Register 40009, Starts the Motor in Forward Rotation

###### `StartReverseRotation`

Register 40009, Starts the Motor in Reverse Rotation

###### `ReadRunningFrequency`

Register 40014, Read the current frequency the motor runs at (RAM)

###### `WriteRunningFrequency`

Register 40014, Write the frequency

###### `ReadMotorStatus`

Read Register 40201, 40202 and 40203 frequency,current and voltage

###### `WriteParameter`

Write "Arbitrary" Parameters

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MitsubishiCS80Requests { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Copy**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Eq**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(request: MitsubishiCS80Requests) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(request: MitsubishiCS80Requests) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **Hash**
  - ```rust
    fn hash<__H: $crate::hash::Hasher>(self: &Self, state: &mut __H) { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &MitsubishiCS80Requests) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

  - ```rust
    fn try_from(value: u32) -> Result<Self, <Self as >::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MitsubishiCS80Status`

```rust
pub struct MitsubishiCS80Status {
    pub running: bool,
    pub forward_running: bool,
    pub reverse_running: bool,
    pub su: bool,
    pub ol: bool,
    pub no_function: bool,
    pub fu: bool,
    pub abc_: bool,
    pub fault_occurence: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `running` | `bool` |  |
| `forward_running` | `bool` |  |
| `reverse_running` | `bool` |  |
| `su` | `bool` |  |
| `ol` | `bool` |  |
| `no_function` | `bool` |  |
| `fu` | `bool` |  |
| `abc_` | `bool` |  |
| `fault_occurence` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> MitsubishiCS80Status { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MotorStatus`

```rust
pub struct MotorStatus {
    pub rpm: uom::si::f64::AngularVelocity,
    pub frequency: uom::si::f64::Frequency,
    pub current: uom::si::f64::ElectricCurrent,
    pub voltage: uom::si::f64::ElectricPotential,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `rpm` | `uom::si::f64::AngularVelocity` |  |
| `frequency` | `uom::si::f64::Frequency` |  |
| `current` | `uom::si::f64::ElectricCurrent` |  |
| `voltage` | `uom::si::f64::ElectricPotential` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MotorStatus { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Copy**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> MotorStatus { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(status: MotorStatus) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MitsubishiCS80`

```rust
pub struct MitsubishiCS80 {
    pub status: MitsubishiCS80Status,
    pub motor_status: MotorStatus,
    pub modbus_serial_interface: control_core::modbus::modbus_serial_interface::ModbusSerialInterface,
    pub last_ts: std::time::Instant,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `status` | `MitsubishiCS80Status` |  |
| `motor_status` | `MotorStatus` |  |
| `modbus_serial_interface` | `control_core::modbus::modbus_serial_interface::ModbusSerialInterface` |  |
| `last_ts` | `std::time::Instant` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(serial_interface: SerialInterface) -> Self { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) fn handle_motor_status(self: &mut Self, resp: &ModbusResponse) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) fn handle_read_inverter_status(self: &mut Self, resp: &ModbusResponse) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) fn handle_response(self: &mut Self, control_request_type: u32) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) fn convert_frequency_to_word(self: &Self, frequency: Frequency) -> u16 { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) fn add_request(self: &mut Self, request: MitsubishiCS80Request) { /* ... */ }
  ```

- ```rust
  pub fn stop_motor(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn set_frequency_target(self: &mut Self, frequency: Frequency) { /* ... */ }
  ```

- ```rust
  pub fn set_rotation(self: &mut Self, forward_rotation: bool) { /* ... */ }
  ```

- ```rust
  pub fn reset_inverter(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub async fn act(self: &mut Self, now: Instant) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `RequestType`

```rust
pub enum RequestType {
    None,
    OperationCommand,
    ReadWrite,
    ParamClear,
    Reset,
}
```

##### Variants

###### `None`

###### `OperationCommand`

Monitoring, Operation (start,stop etc) command, frequency setting (RAM), less than 12 milliseconds timeout for Response

###### `ReadWrite`

Parameter Read/Write and Frequency (EEPROM), Less than 30 milliseconds timeout for Response

###### `ParamClear`

Less than 5 seconds timeout for Response

###### `Reset`

Supposedly no waiting time, however inverter takes a while to start ~300ms should be more than enough

##### Implementations

###### Methods

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) const fn timeout_duration(self: Self) -> Duration { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> RequestType { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Copy**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MitsubishiCS80Request`

```rust
pub struct MitsubishiCS80Request {
    pub(in ::machines::extruder1::mitsubishi_cs80) request: control_core::modbus::ModbusRequest,
    pub(in ::machines::extruder1::mitsubishi_cs80) control_request_type: MitsubishiCS80Requests,
    pub(in ::machines::extruder1::mitsubishi_cs80) request_type: RequestType,
    pub(in ::machines::extruder1::mitsubishi_cs80) priority: u16,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `request` | `control_core::modbus::ModbusRequest` |  |
| `control_request_type` | `MitsubishiCS80Requests` |  |
| `request_type` | `RequestType` |  |
| `priority` | `u16` |  |

##### Implementations

###### Methods

- ```rust
  pub(in ::machines::extruder1::mitsubishi_cs80) const fn new(request: ModbusRequest, control_request_type: MitsubishiCS80Requests, request_type: RequestType, priority: u16) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MitsubishiCS80Request { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(request: MitsubishiCS80Requests) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `new`

```rust
pub mod new { /* ... */ }
```

## Module `screw_speed_controller`

```rust
pub mod screw_speed_controller { /* ... */ }
```

### Types

#### Struct `ScrewSpeedController`

```rust
pub struct ScrewSpeedController {
    pub pid: control_core::controllers::clamping_timeagnostic_pid::ClampingTimeagnosticPidController,
    pub target_pressure: uom::si::f64::Pressure,
    pub target_rpm: uom::si::f64::AngularVelocity,
    pub inverter: super::mitsubishi_cs80::MitsubishiCS80,
    pub(in ::machines::extruder1::screw_speed_controller) pressure_sensor: ethercat_hal::io::analog_input::AnalogInput,
    pub(in ::machines::extruder1::screw_speed_controller) last_update: std::time::Instant,
    pub(in ::machines::extruder1::screw_speed_controller) uses_rpm: bool,
    pub(in ::machines::extruder1::screw_speed_controller) forward_rotation: bool,
    pub(in ::machines::extruder1::screw_speed_controller) transmission: control_core::transmission::fixed::FixedTransmission,
    pub(in ::machines::extruder1::screw_speed_controller) frequency: uom::si::f64::Frequency,
    pub(in ::machines::extruder1::screw_speed_controller) maximum_frequency: uom::si::f64::Frequency,
    pub(in ::machines::extruder1::screw_speed_controller) minimum_frequency: uom::si::f64::Frequency,
    pub(in ::machines::extruder1::screw_speed_controller) motor_on: bool,
    pub(in ::machines::extruder1::screw_speed_controller) nozzle_pressure_limit: uom::si::f64::Pressure,
    pub(in ::machines::extruder1::screw_speed_controller) nozzle_pressure_limit_enabled: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `pid` | `control_core::controllers::clamping_timeagnostic_pid::ClampingTimeagnosticPidController` |  |
| `target_pressure` | `uom::si::f64::Pressure` |  |
| `target_rpm` | `uom::si::f64::AngularVelocity` |  |
| `inverter` | `super::mitsubishi_cs80::MitsubishiCS80` |  |
| `pressure_sensor` | `ethercat_hal::io::analog_input::AnalogInput` |  |
| `last_update` | `std::time::Instant` |  |
| `uses_rpm` | `bool` |  |
| `forward_rotation` | `bool` |  |
| `transmission` | `control_core::transmission::fixed::FixedTransmission` |  |
| `frequency` | `uom::si::f64::Frequency` |  |
| `maximum_frequency` | `uom::si::f64::Frequency` |  |
| `minimum_frequency` | `uom::si::f64::Frequency` |  |
| `motor_on` | `bool` |  |
| `nozzle_pressure_limit` | `uom::si::f64::Pressure` |  |
| `nozzle_pressure_limit_enabled` | `bool` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(inverter: MitsubishiCS80, target_pressure: Pressure, target_rpm: AngularVelocity, pressure_sensor: AnalogInput) -> Self { /* ... */ }
  ```

- ```rust
  pub fn get_motor_enabled(self: &mut Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn set_nozzle_pressure_limit(self: &mut Self, pressure: Pressure) { /* ... */ }
  ```

- ```rust
  pub fn get_nozzle_pressure_limit(self: &mut Self) -> Pressure { /* ... */ }
  ```

- ```rust
  pub fn get_nozzle_pressure_limit_enabled(self: &mut Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn set_nozzle_pressure_limit_is_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

- ```rust
  pub fn get_target_rpm(self: &mut Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn get_rotation_direction(self: &mut Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn set_rotation_direction(self: &mut Self, forward: bool) { /* ... */ }
  ```

- ```rust
  pub fn set_target_pressure(self: &mut Self, target_pressure: Pressure) { /* ... */ }
  ```

- ```rust
  pub fn set_target_screw_rpm(self: &mut Self, target_rpm: AngularVelocity) { /* ... */ }
  ```

- ```rust
  pub fn get_uses_rpm(self: &mut Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn set_uses_rpm(self: &mut Self, uses_rpm: bool) { /* ... */ }
  ```

- ```rust
  pub fn turn_motor_off(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn turn_motor_on(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn get_motor_status(self: &mut Self) -> MotorStatus { /* ... */ }
  ```

- ```rust
  pub fn get_target_pressure(self: &Self) -> Pressure { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1::screw_speed_controller) fn clamp_frequency(frequency: Frequency, min: Frequency, max: Frequency) -> Frequency { /* ... */ }
  ```

- ```rust
  pub fn get_wiring_error(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn get_sensor_current(self: &Self) -> Result<ElectricCurrent, anyhow::Error> { /* ... */ }
  ```

- ```rust
  pub fn reset_pid(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn get_pressure(self: &mut Self) -> Pressure { /* ... */ }
  ```

- ```rust
  pub fn update(self: &mut Self, now: Instant, is_extruding: bool) { /* ... */ }
  ```

- ```rust
  pub fn start_pressure_regulation(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn reset(self: &mut Self) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `temperature_controller`

```rust
pub mod temperature_controller { /* ... */ }
```

### Types

#### Struct `TemperatureController`

```rust
pub struct TemperatureController {
    pub pid: control_core::controllers::pid::PidController,
    pub(in ::machines::extruder1::temperature_controller) temperature_sensor: ethercat_hal::io::temperature_input::TemperatureInput,
    pub(in ::machines::extruder1::temperature_controller) relais: ethercat_hal::io::digital_output::DigitalOutput,
    pub heating: super::Heating,
    pub target_temp: uom::si::f64::ThermodynamicTemperature,
    pub(in ::machines::extruder1::temperature_controller) window_start: std::time::Instant,
    pub(in ::machines::extruder1::temperature_controller) heating_allowed: bool,
    pub(in ::machines::extruder1::temperature_controller) pwm_period: std::time::Duration,
    pub(in ::machines::extruder1::temperature_controller) max_temperature: uom::si::f64::ThermodynamicTemperature,
    pub(in ::machines::extruder1::temperature_controller) temperature_pid_output: f64,
    pub(in ::machines::extruder1::temperature_controller) heating_element_wattage: f64,
    pub(in ::machines::extruder1::temperature_controller) max_clamp: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `pid` | `control_core::controllers::pid::PidController` |  |
| `temperature_sensor` | `ethercat_hal::io::temperature_input::TemperatureInput` |  |
| `relais` | `ethercat_hal::io::digital_output::DigitalOutput` |  |
| `heating` | `super::Heating` |  |
| `target_temp` | `uom::si::f64::ThermodynamicTemperature` |  |
| `window_start` | `std::time::Instant` |  |
| `heating_allowed` | `bool` |  |
| `pwm_period` | `std::time::Duration` |  |
| `max_temperature` | `uom::si::f64::ThermodynamicTemperature` |  |
| `temperature_pid_output` | `f64` |  |
| `heating_element_wattage` | `f64` |  |
| `max_clamp` | `f64` |  |

##### Implementations

###### Methods

- ```rust
  pub fn disable(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn new(kp: f64, ki: f64, kd: f64, target_temp: ThermodynamicTemperature, max_temperature: ThermodynamicTemperature, temperature_sensor: TemperatureInput, relais: DigitalOutput, heating: Heating, pwm_duration: Duration, heating_element_wattage: f64, max_clamp: f64) -> Self { /* ... */ }
  ```

- ```rust
  pub fn set_target_temperature(self: &mut Self, temp: ThermodynamicTemperature) { /* ... */ }
  ```

- ```rust
  pub fn disallow_heating(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn allow_heating(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn get_heating_element_wattage(self: &mut Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn update(self: &mut Self, now: Instant) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Types

#### Enum `ExtruderV2Mode`

```rust
pub enum ExtruderV2Mode {
    Standby,
    Heat,
    Extrude,
}
```

##### Variants

###### `Standby`

###### `Heat`

###### `Extrude`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ExtruderV2Mode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &ExtruderV2Mode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `Heating`

```rust
pub struct Heating {
    pub temperature: uom::si::f64::ThermodynamicTemperature,
    pub heating: bool,
    pub target_temperature: uom::si::f64::ThermodynamicTemperature,
    pub wiring_error: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `temperature` | `uom::si::f64::ThermodynamicTemperature` |  |
| `heating` | `bool` |  |
| `target_temperature` | `uom::si::f64::ThermodynamicTemperature` |  |
| `wiring_error` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> Heating { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> Self { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &Heating) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `HeatingType`

```rust
pub enum HeatingType {
    Nozzle,
    Front,
    Back,
    Middle,
}
```

##### Variants

###### `Nozzle`

###### `Front`

###### `Back`

###### `Middle`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ExtruderV2`

```rust
pub struct ExtruderV2 {
    pub(in ::machines::extruder1) namespace: api::ExtruderV2Namespace,
    pub(in ::machines::extruder1) last_measurement_emit: std::time::Instant,
    pub(in ::machines::extruder1) last_state_event: Option<api::StateEvent>,
    pub(in ::machines::extruder1) mode: ExtruderV2Mode,
    pub(in ::machines::extruder1) screw_speed_controller: screw_speed_controller::ScrewSpeedController,
    pub(in ::machines::extruder1) temperature_controller_front: temperature_controller::TemperatureController,
    pub(in ::machines::extruder1) temperature_controller_middle: temperature_controller::TemperatureController,
    pub(in ::machines::extruder1) temperature_controller_back: temperature_controller::TemperatureController,
    pub(in ::machines::extruder1) temperature_controller_nozzle: temperature_controller::TemperatureController,
    pub(in ::machines::extruder1) emitted_default_state: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `api::ExtruderV2Namespace` |  |
| `last_measurement_emit` | `std::time::Instant` |  |
| `last_state_event` | `Option<api::StateEvent>` |  |
| `mode` | `ExtruderV2Mode` |  |
| `screw_speed_controller` | `screw_speed_controller::ScrewSpeedController` |  |
| `temperature_controller_front` | `temperature_controller::TemperatureController` |  |
| `temperature_controller_middle` | `temperature_controller::TemperatureController` |  |
| `temperature_controller_back` | `temperature_controller::TemperatureController` |  |
| `temperature_controller_nozzle` | `temperature_controller::TemperatureController` |  |
| `emitted_default_state` | `bool` | will be initalized as false and set to true by `emit_state`<br>This way we can signal to the client that the first state emission is a default state |

##### Implementations

###### Methods

- ```rust
  pub fn build_state_event(self: &mut Self) -> StateEvent { /* ... */ }
  ```

- ```rust
  pub fn maybe_emit_state_event(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn emit_live_values(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn emit_state(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_nozzle_pressure_limit_is_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_nozzle_pressure_limit(self: &mut Self, pressure: f64) { /* ... */ }
  ```
  pressure is represented as bar

- ```rust
  pub(in ::machines::extruder1) fn turn_heating_off(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn enable_heating(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn switch_to_standby(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn switch_to_heat(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn switch_to_extrude(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn switch_mode(self: &mut Self, mode: ExtruderV2Mode) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_rotation_state(self: &mut Self, forward: bool) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn reset_inverter(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_mode_state(self: &mut Self, mode: ExtruderV2Mode) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_regulation(self: &mut Self, uses_rpm: bool) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_target_pressure(self: &mut Self, pressure: f64) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_target_rpm(self: &mut Self, rpm: f64) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn set_target_temperature(self: &mut Self, target_temperature: f64, heating_type: HeatingType) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::extruder1) fn configure_pressure_pid(self: &mut Self, settings: PidSettings) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut std::fmt::Formatter<''_>) -> std::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Machine**
  - ```rust
    fn as_any(self: &Self) -> &dyn Any { /* ... */ }
    ```

- **MachineAct**
  - ```rust
    fn act(self: &mut Self, now_ts: Instant) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + ''_>> { /* ... */ }
    ```

- **MachineApi**
  - ```rust
    fn api_mutate(self: &mut Self, request_body: Value) -> Result<(), anyhow::Error> { /* ... */ }
    ```

  - ```rust
    fn api_event_namespace(self: &mut Self) -> &mut Namespace { /* ... */ }
    ```

- **MachineNewTrait**
  - ```rust
    fn new<''maindevice>(params: &MachineNewParams<''_, ''_, ''_, ''_, ''_, ''_, ''_>) -> Result<Self, Error> { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToString**
  - ```rust
    fn to_string(self: &Self) -> String { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `laser`

```rust
pub mod laser { /* ... */ }
```

### Modules

## Module `act`

```rust
pub mod act { /* ... */ }
```

## Module `api`

```rust
pub mod api { /* ... */ }
```

### Types

#### Struct `LiveValuesEvent`

```rust
pub struct LiveValuesEvent {
    pub diameter: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `diameter` | `f64` | diameter measurement in mm |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LiveValuesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> LiveValuesEvent { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `StateEvent`

```rust
pub struct StateEvent {
    pub is_default_state: bool,
    pub laser_state: LaserState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `is_default_state` | `bool` |  |
| `laser_state` | `LaserState` | laser state |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **BuildEvent**
  - ```rust
    fn build(self: &Self) -> control_core::socketio::event::Event<Self> { /* ... */ }
    ```
    Implemented by the BuildEvent derive macro

- **Clone**
  - ```rust
    fn clone(self: &Self) -> StateEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LaserState`

```rust
pub struct LaserState {
    pub higher_tolerance: f64,
    pub lower_tolerance: f64,
    pub target_diameter: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `higher_tolerance` | `f64` | higher tolerance in mm |
| `lower_tolerance` | `f64` | lower tolerance in mm |
| `target_diameter` | `f64` | target diameter in mm |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LaserState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `LaserEvents`

```rust
pub enum LaserEvents {
    LiveValues(control_core::socketio::event::Event<LiveValuesEvent>),
    State(control_core::socketio::event::Event<StateEvent>),
}
```

##### Variants

###### `LiveValues`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<LiveValuesEvent>` |  |

###### `State`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<StateEvent>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **CacheableEvents**
  - ```rust
    fn event_value(self: &Self) -> GenericEvent { /* ... */ }
    ```

  - ```rust
    fn event_cache_fn(self: &Self) -> CacheFn { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: LaserEvents) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LaserMachineNamespace`

```rust
pub struct LaserMachineNamespace {
    pub namespace: control_core::socketio::namespace::Namespace,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `control_core::socketio::namespace::Namespace` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: LaserEvents) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Mutation`

All values in the Mutation enum should be positive.
This ensures that the parameters for setting tolerances and target diameter
are valid and meaningful within the context of the LaserMachine's operation.

```rust
pub(in ::machines::laser::api) enum Mutation {
    SetTargetDiameter(f64),
    SetLowerTolerance(f64),
    SetHigherTolerance(f64),
}
```

##### Variants

###### `SetTargetDiameter`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetLowerTolerance`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetHigherTolerance`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `new`

```rust
pub mod new { /* ... */ }
```

### Types

#### Struct `LaserMachine`

```rust
pub struct LaserMachine {
    pub(in ::machines::laser) laser: std::sync::Arc<smol::lock::RwLock<crate::serial::devices::laser::Laser>>,
    pub(in ::machines::laser) namespace: api::LaserMachineNamespace,
    pub(in ::machines::laser) last_measurement_emit: std::time::Instant,
    pub(in ::machines::laser) laser_target: LaserTarget,
    pub(in ::machines::laser) emitted_default_state: bool,
    pub(in ::machines::laser) last_state_event: Option<api::StateEvent>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `laser` | `std::sync::Arc<smol::lock::RwLock<crate::serial::devices::laser::Laser>>` |  |
| `namespace` | `api::LaserMachineNamespace` |  |
| `last_measurement_emit` | `std::time::Instant` |  |
| `laser_target` | `LaserTarget` |  |
| `emitted_default_state` | `bool` | Will be initialized as false and set to true by emit_state<br>This way we can signal to the client that the first state emission is a default state |
| `last_state_event` | `Option<api::StateEvent>` |  |

##### Implementations

###### Methods

- ```rust
  pub fn emit_live_values(self: &mut Self) { /* ... */ }
  ```
  diameter in mm

- ```rust
  pub fn build_state_event(self: &Self) -> StateEvent { /* ... */ }
  ```

- ```rust
  pub fn maybe_emit_state_event(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn emit_state(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn set_higher_tolerance(self: &mut Self, higher_tolerance: f64) { /* ... */ }
  ```

- ```rust
  pub fn set_lower_tolerance(self: &mut Self, lower_tolerance: f64) { /* ... */ }
  ```

- ```rust
  pub fn set_target_diameter(self: &mut Self, target_diameter: f64) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Machine**
  - ```rust
    fn as_any(self: &Self) -> &dyn Any { /* ... */ }
    ```

- **MachineAct**
  - ```rust
    fn act(self: &mut Self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + ''_>> { /* ... */ }
    ```

- **MachineApi**
  - ```rust
    fn api_mutate(self: &mut Self, request_body: Value) -> Result<(), anyhow::Error> { /* ... */ }
    ```

  - ```rust
    fn api_event_namespace(self: &mut Self) -> &mut Namespace { /* ... */ }
    ```

- **MachineNewTrait**
  - ```rust
    fn new<''maindevice, ''subdevices>(params: &control_core::machines::new::MachineNewParams<''maindevice, ''subdevices, ''_, ''_, ''_, ''_, ''_>) -> Result<Self, Error>
where
    Self: Sized { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LaserTarget`

```rust
pub struct LaserTarget {
    pub(in ::machines::laser) diameter: uom::si::f64::Length,
    pub(in ::machines::laser) lower_tolerance: uom::si::f64::Length,
    pub(in ::machines::laser) higher_tolerance: uom::si::f64::Length,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `diameter` | `uom::si::f64::Length` |  |
| `lower_tolerance` | `uom::si::f64::Length` |  |
| `higher_tolerance` | `uom::si::f64::Length` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LaserTarget { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `mock`

```rust
pub mod mock { /* ... */ }
```

### Modules

## Module `act`

```rust
pub mod act { /* ... */ }
```

## Module `api`

```rust
pub mod api { /* ... */ }
```

### Types

#### Enum `Mode`

```rust
pub enum Mode {
    Standby,
    Running,
}
```

##### Variants

###### `Standby`

###### `Running`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> Mode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &Mode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LiveValuesEvent`

```rust
pub struct LiveValuesEvent {
    pub amplitude_sum: f64,
    pub amplitude1: f64,
    pub amplitude2: f64,
    pub amplitude3: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `amplitude_sum` | `f64` |  |
| `amplitude1` | `f64` |  |
| `amplitude2` | `f64` |  |
| `amplitude3` | `f64` |  |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LiveValuesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> LiveValuesEvent { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `StateEvent`

```rust
pub struct StateEvent {
    pub is_default_state: bool,
    pub frequency1: f64,
    pub frequency2: f64,
    pub frequency3: f64,
    pub mode_state: ModeState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `is_default_state` | `bool` |  |
| `frequency1` | `f64` | sine wave frequencies in millihertz |
| `frequency2` | `f64` |  |
| `frequency3` | `f64` |  |
| `mode_state` | `ModeState` | mode state |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **BuildEvent**
  - ```rust
    fn build(self: &Self) -> control_core::socketio::event::Event<Self> { /* ... */ }
    ```
    Implemented by the BuildEvent derive macro

- **Clone**
  - ```rust
    fn clone(self: &Self) -> StateEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &StateEvent) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ModeState`

```rust
pub struct ModeState {
    pub mode: Mode,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mode` | `Mode` | current mode |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ModeState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &ModeState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `MockEvents`

```rust
pub enum MockEvents {
    LiveValues(control_core::socketio::event::Event<LiveValuesEvent>),
    State(control_core::socketio::event::Event<StateEvent>),
}
```

##### Variants

###### `LiveValues`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<LiveValuesEvent>` |  |

###### `State`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<StateEvent>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **CacheableEvents**
  - ```rust
    fn event_value(self: &Self) -> GenericEvent { /* ... */ }
    ```

  - ```rust
    fn event_cache_fn(self: &Self) -> CacheFn { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: MockEvents) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MockMachineNamespace`

```rust
pub struct MockMachineNamespace {
    pub namespace: control_core::socketio::namespace::Namespace,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `control_core::socketio::namespace::Namespace` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: MockEvents) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Mutation`

Mutation for controlling the mock machine

```rust
pub(in ::machines::mock::api) enum Mutation {
    SetFrequency1(f64),
    SetFrequency2(f64),
    SetFrequency3(f64),
    SetMode(Mode),
}
```

##### Variants

###### `SetFrequency1`

Set the frequency of the sine wave in millihertz

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetFrequency2`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetFrequency3`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetMode`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Mode` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `new`

```rust
pub mod new { /* ... */ }
```

### Types

#### Struct `MockMachine`

```rust
pub struct MockMachine {
    pub(in ::machines::mock) namespace: api::MockMachineNamespace,
    pub(in ::machines::mock) last_measurement_emit: std::time::Instant,
    pub(in ::machines::mock) t_0: std::time::Instant,
    pub(in ::machines::mock) frequency1: uom::si::f64::Frequency,
    pub(in ::machines::mock) frequency2: uom::si::f64::Frequency,
    pub(in ::machines::mock) frequency3: uom::si::f64::Frequency,
    pub(in ::machines::mock) mode: api::Mode,
    pub(in ::machines::mock) last_emitted_event: Option<api::StateEvent>,
    pub(in ::machines::mock) emitted_default_state: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `api::MockMachineNamespace` |  |
| `last_measurement_emit` | `std::time::Instant` |  |
| `t_0` | `std::time::Instant` |  |
| `frequency1` | `uom::si::f64::Frequency` |  |
| `frequency2` | `uom::si::f64::Frequency` |  |
| `frequency3` | `uom::si::f64::Frequency` |  |
| `mode` | `api::Mode` |  |
| `last_emitted_event` | `Option<api::StateEvent>` |  |
| `emitted_default_state` | `bool` | Will be initialized as false and set to true by emit_state<br>This way we can signal to the client that the first state emission is a default state |

##### Implementations

###### Methods

- ```rust
  pub fn emit_live_values(self: &mut Self) { /* ... */ }
  ```
  Emit live values data event with the current sine wave amplitude

- ```rust
  pub fn emit_state(self: &mut Self) { /* ... */ }
  ```
  Emit the current state of the mock machine only if values have changed

- ```rust
  pub fn maybe_emit_state_event(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn set_frequency1(self: &mut Self, frequency_mhz: f64) { /* ... */ }
  ```
  Set the frequencies of the sine waves

- ```rust
  pub fn set_frequency2(self: &mut Self, frequency_mhz: f64) { /* ... */ }
  ```

- ```rust
  pub fn set_frequency3(self: &mut Self, frequency_mhz: f64) { /* ... */ }
  ```

- ```rust
  pub fn set_mode(self: &mut Self, mode: Mode) { /* ... */ }
  ```
  Set the mode of the mock machine

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Machine**
  - ```rust
    fn as_any(self: &Self) -> &dyn std::any::Any { /* ... */ }
    ```

- **MachineAct**
  - ```rust
    fn act(self: &mut Self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + ''_>> { /* ... */ }
    ```

- **MachineApi**
  - ```rust
    fn api_mutate(self: &mut Self, request_body: Value) -> Result<(), anyhow::Error> { /* ... */ }
    ```

  - ```rust
    fn api_event_namespace(self: &mut Self) -> &mut Namespace { /* ... */ }
    ```

- **MachineNewTrait**
  - ```rust
    fn new<''maindevice, ''subdevices>(params: &control_core::machines::new::MachineNewParams<''maindevice, ''subdevices, ''_, ''_, ''_, ''_, ''_>) -> Result<Self, Error>
where
    Self: Sized { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `registry`

```rust
pub mod registry { /* ... */ }
```

### Types

#### Struct `MACHINE_REGISTRY`

**Attributes:**

- `#[allow(missing_copy_implementations)]`
- `#[allow(non_camel_case_types)]`
- `#[allow(dead_code)]`

```rust
pub struct MACHINE_REGISTRY {
    pub(in ::machines::registry) __private_field: (),
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `__private_field` | `()` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deref**
  - ```rust
    fn deref(self: &Self) -> &MachineRegistry { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **LazyStatic**
- **Pipe**
- **Receiver**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `winder2`

```rust
pub mod winder2 { /* ... */ }
```

### Modules

## Module `act`

```rust
pub mod act { /* ... */ }
```

## Module `adaptive_spool_speed_controller`

```rust
pub mod adaptive_spool_speed_controller { /* ... */ }
```

### Types

#### Struct `AdaptiveSpoolSpeedController`

Adaptive spool speed controller that automatically adjusts to maintain optimal filament tension.

This controller monitors filament tension via the tension arm and learns the appropriate
maximum speed based on puller speed and tension feedback. It uses closed-loop control
to minimize tension error and applies smooth acceleration to prevent sudden motor commands.

```rust
pub struct AdaptiveSpoolSpeedController {
    pub(in ::machines::winder2::adaptive_spool_speed_controller) last_speed: uom::si::f64::AngularVelocity,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) enabled: bool,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) acceleration_controller: control_core::controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) filament_calc: crate::machines::winder2::filament_tension::FilamentTensionCalculator,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) speed_time_window: control_core::helpers::moving_time_window::MovingTimeWindow<f64>,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) radius: uom::si::f64::Length,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) last_max_speed_factor_update: Option<std::time::Instant>,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) tension_target: f64,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) radius_learning_rate: f64,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) max_speed_multiplier: f64,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) acceleration_factor: f64,
    pub(in ::machines::winder2::adaptive_spool_speed_controller) deacceleration_urgency_multiplier: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `last_speed` | `uom::si::f64::AngularVelocity` | Last commanded angular velocity sent to the spool motor |
| `enabled` | `bool` | Whether the speed controller is enabled (false = always returns zero speed) |
| `acceleration_controller` | `control_core::controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController` | Acceleration controller to smooth speed transitions and prevent sudden changes |
| `filament_calc` | `crate::machines::winder2::filament_tension::FilamentTensionCalculator` | Calculator for converting tension arm angle to normalized filament tension |
| `speed_time_window` | `control_core::helpers::moving_time_window::MovingTimeWindow<f64>` | Moving window of recent speeds (in rad/s) used for dynamic acceleration limit calculation |
| `radius` | `uom::si::f64::Length` | Estimated diameter in cm |
| `last_max_speed_factor_update` | `Option<std::time::Instant>` | Timestamp of last max speed factor update, used for time-aware learning rate calculation |
| `tension_target` | `f64` | Target normalized tension value (0.0-1.0) that the controller tries to maintain |
| `radius_learning_rate` | `f64` | Proportional control gain for adaptive learning (negative: higher tension reduces speed) |
| `max_speed_multiplier` | `f64` | Speed multiplier when tension is at minimum (max speed factor) |
| `acceleration_factor` | `f64` | Base acceleration as a fraction of max possible speed (per second) |
| `deacceleration_urgency_multiplier` | `f64` | Urgency multiplier for near-zero target speeds |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Creates a new adaptive spool speed controller with default settings.

- ```rust
  pub(in ::machines::winder2::adaptive_spool_speed_controller) fn get_max_speed(self: &Self, puller_speed_controller: &PullerSpeedController) -> AngularVelocity { /* ... */ }
  ```
  Calculates the current maximum speed based on puller speed and learned factor.

- ```rust
  pub(in ::machines::winder2::adaptive_spool_speed_controller) fn calculate_speed(self: &mut Self, t: Instant, tension_arm: &TensionArm, puller_speed_controller: &PullerSpeedController) -> AngularVelocity { /* ... */ }
  ```
  Calculates the desired spool speed based on filament tension feedback.

- ```rust
  pub(in ::machines::winder2::adaptive_spool_speed_controller) fn accelerate_speed(self: &mut Self, target_speed: AngularVelocity, puller_speed_controller: &PullerSpeedController, t: Instant) -> AngularVelocity { /* ... */ }
  ```
  Simplified urgency-weighted acceleration that adapts to current operating conditions.

- ```rust
  pub(in ::machines::winder2::adaptive_spool_speed_controller) fn clamp_speed(self: &mut Self, speed: AngularVelocity) -> AngularVelocity { /* ... */ }
  ```
  Safety function that enforces absolute speed limits.

- ```rust
  pub(in ::machines::winder2::adaptive_spool_speed_controller) fn update_radius(self: &mut Self, filament_tension: f64, t: Instant) { /* ... */ }
  ```
  Adaptive learning algorithm that adjusts maximum speed factor based on tension feedback.

- ```rust
  pub fn update_speed(self: &mut Self, t: Instant, tension_arm: &TensionArm, puller_speed_controller: &PullerSpeedController) -> AngularVelocity { /* ... */ }
  ```
  Main update function that orchestrates the complete speed control pipeline.

- ```rust
  pub fn set_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```
  Enables or disables the speed controller.

- ```rust
  pub fn is_enabled(self: &Self) -> bool { /* ... */ }
  ```
  Returns whether the speed controller is currently enabled.

- ```rust
  pub fn reset(self: &mut Self) { /* ... */ }
  ```
  Resets the controller to initial state, clearing all learned parameters.

- ```rust
  pub fn get_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```
  Returns the last commanded speed from the controller.

- ```rust
  pub fn set_speed(self: &mut Self, speed: AngularVelocity) { /* ... */ }
  ```
  Manually sets the current speed, bypassing normal calculation.

- ```rust
  pub fn get_radius(self: &Self) -> Length { /* ... */ }
  ```

- ```rust
  pub fn get_tension_target(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_tension_target(self: &mut Self, tension_target: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_radius_learning_rate(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_radius_learning_rate(self: &mut Self, radius_learning_rate: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_max_speed_multiplier(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_max_speed_multiplier(self: &mut Self, max_speed_multiplier: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_acceleration_factor(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_acceleration_factor(self: &mut Self, acceleration_factor: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_deacceleration_urgency_multiplier(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_deacceleration_urgency_multiplier(self: &mut Self, deacceleration_urgency_multiplier: f64) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `api`

```rust
pub mod api { /* ... */ }
```

### Types

#### Enum `Mode`

```rust
pub enum Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}
```

##### Variants

###### `Standby`

###### `Hold`

###### `Pull`

###### `Wind`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> Mode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(mode: Mode) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Mutation`

```rust
pub(in ::machines::winder2::api) enum Mutation {
    SetTraverseLimitOuter(f64),
    SetTraverseLimitInner(f64),
    SetTraverseStepSize(f64),
    SetTraversePadding(f64),
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),
    SetPullerRegulationMode(super::puller_speed_controller::PullerRegulationMode),
    SetPullerTargetSpeed(f64),
    SetPullerTargetDiameter(f64),
    SetPullerForward(bool),
    SetSpoolRegulationMode(super::spool_speed_controller::SpoolSpeedControllerType),
    SetSpoolMinMaxMinSpeed(f64),
    SetSpoolMinMaxMaxSpeed(f64),
    SetSpoolAdaptiveTensionTarget(f64),
    SetSpoolAdaptiveRadiusLearningRate(f64),
    SetSpoolAdaptiveMaxSpeedMultiplier(f64),
    SetSpoolAdaptiveAccelerationFactor(f64),
    SetSpoolAdaptiveDeaccelerationUrgencyMultiplier(f64),
    SetSpoolAutomaticRequiredMeters(f64),
    SetSpoolAutomaticAction(SpoolAutomaticActionMode),
    ResetSpoolProgress,
    ZeroTensionArmAngle,
    SetMode(Mode),
    SetConnectedMachine(control_core::machines::identification::MachineIdentificationUnique),
    DisconnectMachine(control_core::machines::identification::MachineIdentificationUnique),
}
```

##### Variants

###### `SetTraverseLimitOuter`

Position in mm from home point

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetTraverseLimitInner`

Position in mm from home point

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetTraverseStepSize`

Step size in mm for traverse movement

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetTraversePadding`

Padding in mm for traverse movement limits

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `GotoTraverseLimitOuter`

###### `GotoTraverseLimitInner`

###### `GotoTraverseHome`

Find home point

###### `EnableTraverseLaserpointer`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `SetPullerRegulationMode`

on = speed, off = stop

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `super::puller_speed_controller::PullerRegulationMode` |  |

###### `SetPullerTargetSpeed`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetPullerTargetDiameter`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetPullerForward`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `SetSpoolRegulationMode`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `super::spool_speed_controller::SpoolSpeedControllerType` |  |

###### `SetSpoolMinMaxMinSpeed`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolMinMaxMaxSpeed`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAdaptiveTensionTarget`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAdaptiveRadiusLearningRate`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAdaptiveMaxSpeedMultiplier`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAdaptiveAccelerationFactor`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAdaptiveDeaccelerationUrgencyMultiplier`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAutomaticRequiredMeters`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `f64` |  |

###### `SetSpoolAutomaticAction`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `SpoolAutomaticActionMode` |  |

###### `ResetSpoolProgress`

###### `ZeroTensionArmAngle`

###### `SetMode`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `Mode` |  |

###### `SetConnectedMachine`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::machines::identification::MachineIdentificationUnique` |  |

###### `DisconnectMachine`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::machines::identification::MachineIdentificationUnique` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LiveValuesEvent`

```rust
pub struct LiveValuesEvent {
    pub traverse_position: Option<f64>,
    pub puller_speed: f64,
    pub spool_rpm: f64,
    pub spool_diameter: f64,
    pub tension_arm_angle: f64,
    pub spool_progress: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `traverse_position` | `Option<f64>` | traverse position in mm |
| `puller_speed` | `f64` | puller speed in m/min |
| `spool_rpm` | `f64` | spool rpm |
| `spool_diameter` | `f64` | spool diameter in mm |
| `tension_arm_angle` | `f64` | tension arm angle in degrees |
| `spool_progress` | `f64` |  |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LiveValuesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Default**
  - ```rust
    fn default() -> LiveValuesEvent { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `StateEvent`

```rust
pub struct StateEvent {
    pub is_default_state: bool,
    pub traverse_state: TraverseState,
    pub puller_state: PullerState,
    pub spool_automatic_action_state: SpoolAutomaticActionState,
    pub mode_state: ModeState,
    pub tension_arm_state: TensionArmState,
    pub spool_speed_controller_state: SpoolSpeedControllerState,
    pub connected_machine_state: ConnectedMachineState,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `is_default_state` | `bool` |  |
| `traverse_state` | `TraverseState` | traverse state |
| `puller_state` | `PullerState` | puller state |
| `spool_automatic_action_state` | `SpoolAutomaticActionState` | spool automatic action state and progress |
| `mode_state` | `ModeState` | mode state |
| `tension_arm_state` | `TensionArmState` | tension arm state |
| `spool_speed_controller_state` | `SpoolSpeedControllerState` | spool speed controller state |
| `connected_machine_state` | `ConnectedMachineState` | connected machine state |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **BuildEvent**
  - ```rust
    fn build(self: &Self) -> control_core::socketio::event::Event<Self> { /* ... */ }
    ```
    Implemented by the BuildEvent derive macro

- **Clone**
  - ```rust
    fn clone(self: &Self) -> StateEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `TraverseState`

```rust
pub struct TraverseState {
    pub limit_inner: f64,
    pub limit_outer: f64,
    pub position_in: f64,
    pub position_out: f64,
    pub is_going_in: bool,
    pub is_going_out: bool,
    pub is_homed: bool,
    pub is_going_home: bool,
    pub is_traversing: bool,
    pub laserpointer: bool,
    pub step_size: f64,
    pub padding: f64,
    pub can_go_in: bool,
    pub can_go_out: bool,
    pub can_go_home: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `limit_inner` | `f64` | min position in mm |
| `limit_outer` | `f64` | max position in mm |
| `position_in` | `f64` | position in mm |
| `position_out` | `f64` | position out in mm |
| `is_going_in` | `bool` | is going to position in |
| `is_going_out` | `bool` | is going to position out |
| `is_homed` | `bool` | if is homed |
| `is_going_home` | `bool` | if is homing |
| `is_traversing` | `bool` | if is traversing |
| `laserpointer` | `bool` | laserpointer is on |
| `step_size` | `f64` | step size in mm |
| `padding` | `f64` | padding in mm |
| `can_go_in` | `bool` | can go in (to inner limit) |
| `can_go_out` | `bool` | can go out (to outer limit) |
| `can_go_home` | `bool` | can home |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> TraverseState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `PullerState`

```rust
pub struct PullerState {
    pub regulation: super::puller_speed_controller::PullerRegulationMode,
    pub target_speed: f64,
    pub target_diameter: f64,
    pub forward: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `regulation` | `super::puller_speed_controller::PullerRegulationMode` | regulation type |
| `target_speed` | `f64` | target speed in m/min |
| `target_diameter` | `f64` | target diameter in mm |
| `forward` | `bool` | forward rotation direction |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PullerState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `SpoolAutomaticActionMode`

```rust
pub enum SpoolAutomaticActionMode {
    NoAction,
    Pull,
    Hold,
}
```

##### Variants

###### `NoAction`

###### `Pull`

###### `Hold`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> SpoolAutomaticActionMode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `SpoolAutomaticActionState`

```rust
pub struct SpoolAutomaticActionState {
    pub spool_required_meters: f64,
    pub spool_automatic_action_mode: SpoolAutomaticActionMode,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `spool_required_meters` | `f64` |  |
| `spool_automatic_action_mode` | `SpoolAutomaticActionMode` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> SpoolAutomaticActionState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ModeState`

```rust
pub struct ModeState {
    pub mode: Mode,
    pub can_wind: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `mode` | `Mode` | mode |
| `can_wind` | `bool` | can wind |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ModeState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `TensionArmState`

```rust
pub struct TensionArmState {
    pub zeroed: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `zeroed` | `bool` | is zeroed |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> TensionArmState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `SpoolSpeedControllerState`

```rust
pub struct SpoolSpeedControllerState {
    pub regulation_mode: super::spool_speed_controller::SpoolSpeedControllerType,
    pub minmax_min_speed: f64,
    pub minmax_max_speed: f64,
    pub adaptive_tension_target: f64,
    pub adaptive_radius_learning_rate: f64,
    pub adaptive_max_speed_multiplier: f64,
    pub adaptive_acceleration_factor: f64,
    pub adaptive_deacceleration_urgency_multiplier: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `regulation_mode` | `super::spool_speed_controller::SpoolSpeedControllerType` | regulation mode |
| `minmax_min_speed` | `f64` | min speed in rpm for minmax mode |
| `minmax_max_speed` | `f64` | max speed in rpm for minmax mode |
| `adaptive_tension_target` | `f64` | tension target for adaptive mode (0.0-1.0) |
| `adaptive_radius_learning_rate` | `f64` | radius learning rate for adaptive mode |
| `adaptive_max_speed_multiplier` | `f64` | max speed multiplier for adaptive mode |
| `adaptive_acceleration_factor` | `f64` | acceleration factor for adaptive mode |
| `adaptive_deacceleration_urgency_multiplier` | `f64` | deacceleration urgency multiplier for adaptive mode |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> SpoolSpeedControllerState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `ConnectedMachineState`

```rust
pub struct ConnectedMachineState {
    pub machine_identification_unique: Option<control_core::machines::identification::MachineIdentificationUnique>,
    pub is_available: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `machine_identification_unique` | `Option<control_core::machines::identification::MachineIdentificationUnique>` | Connected Machine |
| `is_available` | `bool` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> ConnectedMachineState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Winder2Events`

```rust
pub enum Winder2Events {
    LiveValues(control_core::socketio::event::Event<LiveValuesEvent>),
    State(control_core::socketio::event::Event<StateEvent>),
}
```

##### Variants

###### `LiveValues`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<LiveValuesEvent>` |  |

###### `State`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<StateEvent>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **CacheableEvents**
  - ```rust
    fn event_value(self: &Self) -> GenericEvent { /* ... */ }
    ```

  - ```rust
    fn event_cache_fn(self: &Self) -> CacheFn { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: Winder2Events) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `Winder2Namespace`

```rust
pub struct Winder2Namespace {
    pub namespace: control_core::socketio::namespace::Namespace,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `control_core::socketio::namespace::Namespace` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, events: Winder2Events) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `clamp_revolution`

```rust
pub mod clamp_revolution { /* ... */ }
```

### Types

#### Enum `Clamping`

```rust
pub enum Clamping {
    None,
    Min,
    Max,
}
```

##### Variants

###### `None`

###### `Min`

###### `Max`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> Clamping { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Copy**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &Clamping) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Functions

#### Function `clamp_revolution_uom`

Clamps a UOM angle value to be within the specified range [min, max].

This is a wrapper around [`clamp_revolution`] that works with UOM Angle types.

# Arguments

* `value` - The angle value to clamp
* `min` - The minimum acceptable angle
* `max` - The maximum acceptable angle

# Returns

A clamped angle value according to the same rules as [`clamp_revolution`].

# Examples

```ignore
use uom::si::{angle::revolution, f64::Angle};

let value = Angle::new::<revolution>(0.15);
let min = Angle::new::<revolution>(0.1);
let max = Angle::new::<revolution>(0.2);

let clamped = clamp_revolution_uom(value, min, max);
assert_eq!(clamped.get::<revolution>(), 0.15); // Value within range stays the same
```

```rust
pub fn clamp_revolution_uom(value: uom::si::f64::Angle, min: uom::si::f64::Angle, max: uom::si::f64::Angle) -> (uom::si::f64::Angle, Clamping) { /* ... */ }
```

#### Function `scale_revolution_to_range`

Linearly scales a value relative to a specified range.

This function maps the [min, max] range to [0, 1], with min mapping to 0
and max mapping to 1. Values outside the range will map to values outside [0, 1].

Note: Unlike [`clamp_revolution`], this function doesn't clamp values; it performs
a linear scaling even for values outside the range.

# Arguments

* `value` - The value to scale
* `min` - The value that should map to 0
* `max` - The value that should map to 1

# Returns

A linearly scaled value where:
* `value = min` returns 0
* `value = max` returns 1
* Values in between are linearly interpolated
* Values outside the range extrapolate to values outside [0, 1]

# Examples

```ignore
assert_eq!(scale_revolution_to_range(0.3, 0.2, 0.6), 0.25); // 25% between min and max
assert_eq!(scale_revolution_to_range(0.5, 0.4, 0.6), 0.5);  // Midpoint
assert_eq!(scale_revolution_to_range(0.1, 0.2, 0.6), -0.25); // Value below min
```

```rust
pub fn scale_revolution_to_range(value: f64, min: f64, max: f64) -> f64 { /* ... */ }
```

#### Function `clamp_revolution`

Clamps a revolution value to be within the specified range [min, max].

If the value is outside the range, it will be clamped to either min or max,
depending on which one it's closer to in the circular context.

# Arguments

* `value` - The value to clamp
* `min` - The minimum acceptable value
* `max` - The maximum acceptable value

# Returns

* The original value if it's within the range + false, false
* The min value if it's closer to min + true, false
* The max value if it's closer to max + false, true

The first bool indicates if the value was clamped to min,
and the second bool indicates if it was clamped to max.

# Examples

```ignore
// Value within range stays the same
assert_eq!(clamp_revolution(0.15, 0.1, 0.2), 0.15);

// Value outside range gets clamped
assert_eq!(clamp_revolution(0.05, 0.1, 0.2), 0.1);  // Clamped to min
assert_eq!(clamp_revolution(0.25, 0.1, 0.2), 0.2);  // Clamped to max

// With a range that crosses zero
assert_eq!(clamp_revolution(0.5, 0.9, 0.1), 0.9);   // Clamped to min
```

```rust
pub fn clamp_revolution(value: f64, min: f64, max: f64) -> (f64, Clamping) { /* ... */ }
```

#### Function `clamping_ranges`

Calculates the clamping ranges for min and max values in a circular context.

This is used internally by `clamp_revolution` to determine whether out-of-range
values should be clamped to the min or max value.

# Returns

A tuple containing:
* `clamp_to_min_min` - Lower bound of the range for values that should clamp to min
* `clamp_to_min_max` - Upper bound of the range for values that should clamp to min
* `clamp_to_max_min` - Lower bound of the range for values that should clamp to max
* `clamp_to_max_max` - Upper bound of the range for values that should clamp to max

The clamping strategy divides the out-of-range space into two regions:
values closer to min are clamped to min, and values closer to max are clamped to max.

```rust
pub(in ::machines::winder2::clamp_revolution) fn clamping_ranges(min: f64, max: f64) -> (f64, f64, f64, f64) { /* ... */ }
```

#### Function `revolution_distance`

Calculates the shortest distance between two points in a circular [0,1) range.

This function properly handles cases where the shortest path crosses the 0/1 boundary.

# Arguments

* `min` - The first point in the range [0,1)
* `max` - The second point in the range [0,1)

# Examples

```ignore
// Regular distance
assert_eq!(revolution_distance(0.1, 0.3), 0.2);

// Distance that crosses the 0/1 boundary
assert_eq!(revolution_distance(0.9, 0.1), 0.2); // The shortest path crosses zero
```

```rust
pub(in ::machines::winder2::clamp_revolution) fn revolution_distance(min: f64, max: f64) -> f64 { /* ... */ }
```

#### Function `wrap_revolution`

Wraps any floating-point value to the [0,1) range, handling the circular nature of revolutions.

This is useful for normalizing angles or other periodic values that represent
a full revolution when they reach 1.0.

# Examples

```ignore
assert_eq!(wrap_revolution(0.5), 0.5);    // Value within range stays the same
assert_eq!(wrap_revolution(1.5), 0.5);    // 1.5 revolutions = 0.5 of a revolution
assert_eq!(wrap_revolution(-0.25), 0.75); // -0.25 revolutions = 0.75 of a revolution
assert_eq!(wrap_revolution(1.0), 1.0);    // Exactly 1.0 stays as 1.0
```

```rust
pub(in ::machines::winder2::clamp_revolution) fn wrap_revolution(value: f64) -> f64 { /* ... */ }
```

#### Function `revolution_in_range`

Checks if a value is within a specified range in a circular [0,1) context.

This function properly handles ranges that cross the 0/1 boundary.

# Arguments

* `value` - The value to check
* `min` - The lower bound of the range
* `max` - The upper bound of the range

# Returns

* `true` if the value is within the range
* `false` otherwise

# Examples

```ignore
// Regular range
assert_eq!(revolution_in_range(0.15, 0.1, 0.2), true);

// Range that crosses zero
assert_eq!(revolution_in_range(0.95, 0.9, 0.1), true);
assert_eq!(revolution_in_range(0.05, 0.9, 0.1), true);
assert_eq!(revolution_in_range(0.5, 0.9, 0.1), false);
```

```rust
pub(in ::machines::winder2::clamp_revolution) fn revolution_in_range(value: f64, min: f64, max: f64) -> bool { /* ... */ }
```

## Module `filament_tension`

```rust
pub mod filament_tension { /* ... */ }
```

### Types

#### Struct `FilamentTensionCalculator`

The "tension" of the filament is not linear regarding the angle of the tension arm since it moves in an angular motion.
With this calculator we can calculate the filament length and tension based on the angle of the tension arm using geometry.

⠉⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇ 0.0
⠀⠀⠀⠉⠢⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠑⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠈⠒⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠒⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠑⠤⢄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠒⠤⢄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠑⠒⠤⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡇
⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠒⠒⠤⠤⢄⣀⣀⡀⠀⠀⠀⠀⠀⠀⡇
⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠀⠁⠈⠈⠉⠉⠉⠉⠉⠉⠁ 1.0
10.0                                                    90.0

```rust
pub struct FilamentTensionCalculator {
    pub(in ::machines::winder2::filament_tension) point_puller: euclid::Point2D<f64, ()>,
    pub(in ::machines::winder2::filament_tension) tension_arm_origin: euclid::Point2D<f64, ()>,
    pub(in ::machines::winder2::filament_tension) traverse_point: euclid::Point2D<f64, ()>,
    pub(in ::machines::winder2::filament_tension) arm_length: f64,
    pub(in ::machines::winder2::filament_tension) min_angle: uom::si::f64::Angle,
    pub(in ::machines::winder2::filament_tension) max_angle: uom::si::f64::Angle,
    pub(in ::machines::winder2::filament_tension) min_distance: uom::si::f64::Length,
    pub(in ::machines::winder2::filament_tension) max_distance: uom::si::f64::Length,
    pub angle_converter: control_core::converters::angle_converter::AngleConverterUom,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `point_puller` | `euclid::Point2D<f64, ()>` |  |
| `tension_arm_origin` | `euclid::Point2D<f64, ()>` |  |
| `traverse_point` | `euclid::Point2D<f64, ()>` |  |
| `arm_length` | `f64` |  |
| `min_angle` | `uom::si::f64::Angle` | In Y-Flipped CW rotation system |
| `max_angle` | `uom::si::f64::Angle` | In Y-Flipped CW rotation system |
| `min_distance` | `uom::si::f64::Length` |  |
| `max_distance` | `uom::si::f64::Length` |  |
| `angle_converter` | `control_core::converters::angle_converter::AngleConverterUom` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(min_angle: Angle, max_angle: Angle) -> Self { /* ... */ }
  ```
  [`max_angle_deg`] in Y-Flipped CW roation system

- ```rust
  pub fn calc_filament_length(self: &Self, tension_arm_angle: Angle) -> Length { /* ... */ }
  ```
  Calculate the filament length for a given tension arm angle

- ```rust
  pub fn calc_filament_tension(self: &Self, tension_arm_angle: Angle) -> f64 { /* ... */ }
  ```
  Calculate the filament buffer as a value between 0.0 (min) and 1.0 (max)

- ```rust
  pub fn get_min_angle(self: &Self) -> Angle { /* ... */ }
  ```
  Get the optimal angle (minimum filament length)

- ```rust
  pub fn get_max_angle(self: &Self) -> Angle { /* ... */ }
  ```
  Get the maximum reference angle

- ```rust
  pub fn get_min_distance(self: &Self) -> Length { /* ... */ }
  ```
  Get the minimum filament distance

- ```rust
  pub fn get_max_distance(self: &Self) -> Length { /* ... */ }
  ```
  Get the maximum filament distance

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> FilamentTensionCalculator { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `minmax_spool_speed_controller`

```rust
pub mod minmax_spool_speed_controller { /* ... */ }
```

### Types

#### Struct `MinMaxSpoolSpeedController`

```rust
pub struct MinMaxSpoolSpeedController {
    pub(in ::machines::winder2::minmax_spool_speed_controller) last_speed: uom::si::f64::AngularVelocity,
    pub(in ::machines::winder2::minmax_spool_speed_controller) enabled: bool,
    pub(in ::machines::winder2::minmax_spool_speed_controller) acceleration_controller: control_core::controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController,
    pub(in ::machines::winder2::minmax_spool_speed_controller) filament_calc: crate::machines::winder2::filament_tension::FilamentTensionCalculator,
    pub(in ::machines::winder2::minmax_spool_speed_controller) speed_time_window: control_core::helpers::moving_time_window::MovingTimeWindow<f64>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `last_speed` | `uom::si::f64::AngularVelocity` | Current speed in |
| `enabled` | `bool` | Whether the speed controller is enabled or not |
| `acceleration_controller` | `control_core::controllers::first_degree_motion::angular_acceleration_speed_controller::AngularAccelerationSpeedController` | Acceleration controller to dampen speed change |
| `filament_calc` | `crate::machines::winder2::filament_tension::FilamentTensionCalculator` | Filament tension calculator |
| `speed_time_window` | `control_core::helpers::moving_time_window::MovingTimeWindow<f64>` | Unit is angular velocity in rad/s |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Parameters:

- ```rust
  pub(in ::machines::winder2::minmax_spool_speed_controller) fn min_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```
  Helper method to get min speed without Option type

- ```rust
  pub(in ::machines::winder2::minmax_spool_speed_controller) fn max_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```
  Helper method to get max speed without Option type  

- ```rust
  pub(in ::machines::winder2::minmax_spool_speed_controller) fn speed_raw(self: &mut Self, _t: Instant, tension_arm: &TensionArm) -> AngularVelocity { /* ... */ }
  ```
  Calculates the desired speed based on the tension arm angle.

- ```rust
  pub(in ::machines::winder2::minmax_spool_speed_controller) fn accelerate_speed(self: &mut Self, speed: AngularVelocity, t: Instant) -> AngularVelocity { /* ... */ }
  ```
  Accelerates the speed using the acceleration controller.

- ```rust
  pub(in ::machines::winder2::minmax_spool_speed_controller) fn clamp_speed(self: &mut Self, speed: AngularVelocity) -> AngularVelocity { /* ... */ }
  ```
  Clamps the speed to the defined minimum and maximum speed.

- ```rust
  pub fn update_speed(self: &mut Self, t: Instant, tension_arm: &TensionArm) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn set_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

- ```rust
  pub fn is_enabled(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn reset(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::minmax_spool_speed_controller) fn update_acceleration(self: &mut Self) -> Result<(), MotionControllerError> { /* ... */ }
  ```

- ```rust
  pub fn set_max_speed(self: &mut Self, max_speed: AngularVelocity) -> Result<(), MotionControllerError> { /* ... */ }
  ```

- ```rust
  pub fn set_min_speed(self: &mut Self, min_speed: AngularVelocity) -> Result<(), MotionControllerError> { /* ... */ }
  ```

- ```rust
  pub fn get_max_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn get_min_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn get_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn set_speed(self: &mut Self, speed: AngularVelocity) { /* ... */ }
  ```

- ```rust
  pub fn get_radius(self: &Self, puller_speed_controller: &PullerSpeedController) -> Length { /* ... */ }
  ```
  derive the radius from the puller speed and the current angular speed

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `new`

```rust
pub mod new { /* ... */ }
```

## Module `puller_speed_controller`

```rust
pub mod puller_speed_controller { /* ... */ }
```

### Types

#### Struct `PullerSpeedController`

```rust
pub struct PullerSpeedController {
    pub(in ::machines::winder2::puller_speed_controller) enabled: bool,
    pub target_speed: uom::si::f64::Velocity,
    pub target_diameter: uom::si::f64::Length,
    pub regulation_mode: PullerRegulationMode,
    pub forward: bool,
    pub(in ::machines::winder2::puller_speed_controller) acceleration_controller: control_core::controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController,
    pub converter: control_core::converters::linear_step_converter::LinearStepConverter,
    pub last_speed: uom::si::f64::Velocity,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `enabled` | `bool` |  |
| `target_speed` | `uom::si::f64::Velocity` |  |
| `target_diameter` | `uom::si::f64::Length` |  |
| `regulation_mode` | `PullerRegulationMode` |  |
| `forward` | `bool` | Forward rotation direction. If false, applies negative sign to speed |
| `acceleration_controller` | `control_core::controllers::second_degree_motion::linear_jerk_speed_controller::LinearJerkSpeedController` | Linear acceleration controller to dampen speed change |
| `converter` | `control_core::converters::linear_step_converter::LinearStepConverter` | Converter for linear to angular transformations |
| `last_speed` | `uom::si::f64::Velocity` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(target_speed: Velocity, target_diameter: Length, converter: LinearStepConverter) -> Self { /* ... */ }
  ```

- ```rust
  pub fn set_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

- ```rust
  pub fn set_target_speed(self: &mut Self, target: Velocity) { /* ... */ }
  ```

- ```rust
  pub fn set_target_diameter(self: &mut Self, target: Length) { /* ... */ }
  ```

- ```rust
  pub fn set_regulation_mode(self: &mut Self, regulation: PullerRegulationMode) { /* ... */ }
  ```

- ```rust
  pub fn set_forward(self: &mut Self, forward: bool) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::puller_speed_controller) fn update_speed(self: &mut Self, t: Instant) -> Velocity { /* ... */ }
  ```

- ```rust
  pub fn speed_to_angular_velocity(self: &Self, speed: Velocity) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn angular_velocity_to_speed(self: &Self, angular_speed: AngularVelocity) -> Velocity { /* ... */ }
  ```

- ```rust
  pub fn calc_angular_velocity(self: &mut Self, t: Instant) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn get_target_speed(self: &Self) -> Velocity { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `PullerRegulationMode`

```rust
pub enum PullerRegulationMode {
    Speed,
    Diameter,
}
```

##### Variants

###### `Speed`

###### `Diameter`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PullerRegulationMode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `spool_speed_controller`

```rust
pub mod spool_speed_controller { /* ... */ }
```

### Types

#### Enum `SpoolSpeedControllerType`

```rust
pub enum SpoolSpeedControllerType {
    Adaptive,
    MinMax,
}
```

##### Variants

###### `Adaptive`

###### `MinMax`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> SpoolSpeedControllerType { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `SpoolSpeedController`

```rust
pub struct SpoolSpeedController {
    pub(in ::machines::winder2::spool_speed_controller) adaptive_controller: crate::machines::winder2::adaptive_spool_speed_controller::AdaptiveSpoolSpeedController,
    pub(in ::machines::winder2::spool_speed_controller) minmax_controller: crate::machines::winder2::minmax_spool_speed_controller::MinMaxSpoolSpeedController,
    pub(in ::machines::winder2::spool_speed_controller) type: SpoolSpeedControllerType,
    pub(in ::machines::winder2::spool_speed_controller) radius_history: control_core::helpers::moving_time_window::MovingTimeWindow<f64>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `adaptive_controller` | `crate::machines::winder2::adaptive_spool_speed_controller::AdaptiveSpoolSpeedController` |  |
| `minmax_controller` | `crate::machines::winder2::minmax_spool_speed_controller::MinMaxSpoolSpeedController` |  |
| `type` | `SpoolSpeedControllerType` |  |
| `radius_history` | `control_core::helpers::moving_time_window::MovingTimeWindow<f64>` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```

- ```rust
  pub fn get_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn set_speed(self: &mut Self, speed: AngularVelocity) { /* ... */ }
  ```

- ```rust
  pub fn set_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

- ```rust
  pub fn is_enabled(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn set_type(self: &mut Self, type: SpoolSpeedControllerType) { /* ... */ }
  ```

- ```rust
  pub fn get_type(self: &Self) -> &SpoolSpeedControllerType { /* ... */ }
  ```

- ```rust
  pub fn set_minmax_min_speed(self: &mut Self, min_speed: AngularVelocity) -> Result<(), MotionControllerError> { /* ... */ }
  ```

- ```rust
  pub fn set_minmax_max_speed(self: &mut Self, max_speed: AngularVelocity) -> Result<(), MotionControllerError> { /* ... */ }
  ```

- ```rust
  pub fn get_minmax_min_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn get_minmax_max_speed(self: &Self) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn update_speed(self: &mut Self, t: Instant, tension_arm: &TensionArm, puller_speed_controller: &PullerSpeedController) -> AngularVelocity { /* ... */ }
  ```

- ```rust
  pub fn get_estimated_radius(self: &mut Self) -> Length { /* ... */ }
  ```

- ```rust
  pub fn get_adaptive_tension_target(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_adaptive_tension_target(self: &mut Self, tension_target: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_adaptive_radius_learning_rate(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_adaptive_radius_learning_rate(self: &mut Self, radius_learning_rate: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_adaptive_max_speed_multiplier(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_adaptive_max_speed_multiplier(self: &mut Self, max_speed_multiplier: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_adaptive_acceleration_factor(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_adaptive_acceleration_factor(self: &mut Self, acceleration_factor: f64) { /* ... */ }
  ```

- ```rust
  pub fn get_adaptive_deacceleration_urgency_multiplier(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub fn set_adaptive_deacceleration_urgency_multiplier(self: &mut Self, deacceleration_urgency_multiplier: f64) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `tension_arm`

```rust
pub mod tension_arm { /* ... */ }
```

### Types

#### Struct `TensionArm`

```rust
pub struct TensionArm {
    pub analog_input: ethercat_hal::io::analog_input::AnalogInput,
    pub zero: uom::si::f64::Angle,
    pub zeroed: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `analog_input` | `ethercat_hal::io::analog_input::AnalogInput` |  |
| `zero` | `uom::si::f64::Angle` |  |
| `zeroed` | `bool` | was zeroed at least once |

##### Implementations

###### Methods

- ```rust
  pub fn new(analog_input: AnalogInput) -> Self { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::tension_arm) fn volts_to_angle(self: &Self, volts: f64) -> Angle { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::tension_arm) fn get_volts(self: &Self) -> f64 { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::tension_arm) fn raw_angle(self: &Self) -> Angle { /* ... */ }
  ```

- ```rust
  pub fn get_angle(self: &Self) -> Angle { /* ... */ }
  ```

- ```rust
  pub fn zero(self: &mut Self) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `traverse_controller`

```rust
pub mod traverse_controller { /* ... */ }
```

### Types

#### Struct `TraverseController`

```rust
pub struct TraverseController {
    pub(in ::machines::winder2::traverse_controller) enabled: bool,
    pub(in ::machines::winder2::traverse_controller) position: uom::si::f64::Length,
    pub(in ::machines::winder2::traverse_controller) limit_inner: uom::si::f64::Length,
    pub(in ::machines::winder2::traverse_controller) limit_outer: uom::si::f64::Length,
    pub(in ::machines::winder2::traverse_controller) step_size: uom::si::f64::Length,
    pub(in ::machines::winder2::traverse_controller) padding: uom::si::f64::Length,
    pub(in ::machines::winder2::traverse_controller) state: State,
    pub(in ::machines::winder2::traverse_controller) fullstep_converter: control_core::converters::linear_step_converter::LinearStepConverter,
    pub(in ::machines::winder2::traverse_controller) microstep_converter: control_core::converters::linear_step_converter::LinearStepConverter,
    pub(in ::machines::winder2::traverse_controller) did_change_state: bool,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `enabled` | `bool` |  |
| `position` | `uom::si::f64::Length` |  |
| `limit_inner` | `uom::si::f64::Length` |  |
| `limit_outer` | `uom::si::f64::Length` |  |
| `step_size` | `uom::si::f64::Length` |  |
| `padding` | `uom::si::f64::Length` |  |
| `state` | `State` |  |
| `fullstep_converter` | `control_core::converters::linear_step_converter::LinearStepConverter` |  |
| `microstep_converter` | `control_core::converters::linear_step_converter::LinearStepConverter` |  |
| `did_change_state` | `bool` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(limit_inner: Length, limit_outer: Length, microsteps: u8) -> Self { /* ... */ }
  ```

- ```rust
  pub fn set_enabled(self: &mut Self, enabled: bool) { /* ... */ }
  ```

- ```rust
  pub fn set_limit_inner(self: &mut Self, limit: Length) { /* ... */ }
  ```

- ```rust
  pub fn set_limit_outer(self: &mut Self, limit: Length) { /* ... */ }
  ```

- ```rust
  pub fn set_step_size(self: &mut Self, step_size: Length) { /* ... */ }
  ```

- ```rust
  pub fn set_padding(self: &mut Self, padding: Length) { /* ... */ }
  ```

- ```rust
  pub fn get_limit_inner(self: &Self) -> Length { /* ... */ }
  ```

- ```rust
  pub fn get_limit_outer(self: &Self) -> Length { /* ... */ }
  ```

- ```rust
  pub fn get_step_size(self: &Self) -> Length { /* ... */ }
  ```

- ```rust
  pub fn get_padding(self: &Self) -> Length { /* ... */ }
  ```

- ```rust
  pub fn get_current_position(self: &Self) -> Option<Length> { /* ... */ }
  ```

- ```rust
  pub fn is_enabled(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn did_change_state(self: &mut Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn goto_limit_inner(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn goto_limit_outer(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn goto_home(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn start_traversing(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn is_homed(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn is_going_in(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn is_going_out(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn is_going_home(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub fn is_traversing(self: &Self) -> bool { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::traverse_controller) fn is_at_position(self: &Self, target_position: Length, tolerance: Length) -> bool { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2::traverse_controller) fn distance_to_position(self: &Self, target_position: Length) -> Length { /* ... */ }
  ```
  Calculate distance to position

- ```rust
  pub(in ::machines::winder2::traverse_controller) fn speed_to_position(self: &Self, target_position: Length, absolute_speed: Velocity) -> Velocity { /* ... */ }
  ```

- ```rust
  pub fn sync_position(self: &mut Self, traverse: &StepperVelocityEL70x1) { /* ... */ }
  ```
  Gets the current traverse position as a [`Length`].

- ```rust
  pub(in ::machines::winder2::traverse_controller) fn update_did_change_state(self: &mut Self, old_state: &State) -> bool { /* ... */ }
  ```
  Update the [`did_change_state`] flag

- ```rust
  pub(in ::machines::winder2::traverse_controller) fn get_speed(self: &mut Self, traverse: &mut StepperVelocityEL70x1, traverse_end_stop: &DigitalInput, spool_speed: AngularVelocity) -> Velocity { /* ... */ }
  ```
  Calculates a desired speed based on the current state and the end stop status.

- ```rust
  pub fn calculate_traverse_speed(spool_speed: AngularVelocity, step_size: Length) -> Velocity { /* ... */ }
  ```
  Calculate the traverse speed

- ```rust
  pub fn update_speed(self: &mut Self, traverse: &mut StepperVelocityEL70x1, traverse_end_stop: &DigitalInput, spool_speed: AngularVelocity) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `State`

```rust
pub enum State {
    NotHomed,
    Idle,
    GoingIn,
    GoingOut,
    Homing(HomingState),
    Traversing(TraversingState),
}
```

##### Variants

###### `NotHomed`

Initial state

###### `Idle`

Doing nothing
Already homed

###### `GoingIn`

Going to inner limit

After reaching the inner limit, the state will change to [`State::Idle`]

###### `GoingOut`

Going to outer limit

After reaching the outer limit, the state will change to [`State::Idle`]

###### `Homing`

Homing is in progress

After homing is done, the state will change to [`State::Idle`]

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `HomingState` |  |

###### `Traversing`

Move between inner and outer limits

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `TraversingState` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> State { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Eq**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &State) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `TraversingState`

```rust
pub enum TraversingState {
    GoingOut,
    TraversingIn,
    TraversingOut,
}
```

##### Variants

###### `GoingOut`

Like [`State::GoingOut`] but
- will go into [`State::GoingIn`] after reaching the outer limit

###### `TraversingIn`

Like [`State::GoingIn`] but
- will go into [`State::GoingOut`] after reaching the inner limit
- speed is synced to spool speed

###### `TraversingOut`

Like [`State::GoingOut`] but
- will go into [`State::GoingIn`] after reaching the outer limit
- speed is synced to spool speed

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> TraversingState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Eq**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &TraversingState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `HomingState`

```rust
pub enum HomingState {
    Initialize,
    EscapeEndstop,
    FindEndstopFineDistancing,
    FindEndstopCoarse,
    FindEndtopFine,
    Validate(std::time::Instant),
}
```

##### Variants

###### `Initialize`

In this state the traverse is not moving but checks if the endstop si triggered
If the endstop is triggered we go into [`HomingState::EscapeEndstop`]
If the endstop is not triggered we go into [`HomingState::FindEndstop`]

###### `EscapeEndstop`

In this state the traverse is moving out away from the endstop until it's not triggered anymore
The it goes into [`HomingState::FindEnstopFineDistancing`]

###### `FindEndstopFineDistancing`

Moving out away from the endstop
Then Transition into [`HomingState::FindEndtopFine`]

###### `FindEndstopCoarse`

In this state the traverse is fast until it reaches the endstop

###### `FindEndtopFine`

In this state the traverse is moving slowly until it reaches the endstop

###### `Validate`

In this state we check if th current position is actually 0.0, if not we redo the homing routine

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `std::time::Instant` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> HomingState { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Eq**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &HomingState) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Types

#### Struct `SpoolAutomaticAction`

```rust
pub struct SpoolAutomaticAction {
    pub progress: uom::si::f64::Length,
    pub(in ::machines::winder2) progress_last_check: std::time::Instant,
    pub target_length: uom::si::f64::Length,
    pub mode: api::SpoolAutomaticActionMode,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `progress` | `uom::si::f64::Length` |  |
| `progress_last_check` | `std::time::Instant` |  |
| `target_length` | `uom::si::f64::Length` |  |
| `mode` | `api::SpoolAutomaticActionMode` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `Winder2`

```rust
pub struct Winder2 {
    pub traverse: ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1,
    pub puller: ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1,
    pub spool: ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1,
    pub tension_arm: tension_arm::TensionArm,
    pub laser: ethercat_hal::io::digital_output::DigitalOutput,
    pub traverse_controller: traverse_controller::TraverseController,
    pub traverse_end_stop: ethercat_hal::io::digital_input::DigitalInput,
    pub(in ::machines::winder2) namespace: api::Winder2Namespace,
    pub(in ::machines::winder2) last_measurement_emit: std::time::Instant,
    pub machine_manager: std::sync::Weak<smol::lock::RwLock<control_core::machines::manager::MachineManager>>,
    pub machine_identification_unique: control_core::machines::identification::MachineIdentificationUnique,
    pub connected_buffer: Option<control_core::machines::ConnectedMachine<std::sync::Weak<smol::lock::Mutex<crate::machines::buffer1::BufferV1>>>>,
    pub mode: Winder2Mode,
    pub spool_mode: SpoolMode,
    pub traverse_mode: TraverseMode,
    pub puller_mode: PullerMode,
    pub spool_speed_controller: spool_speed_controller::SpoolSpeedController,
    pub spool_step_converter: control_core::converters::angular_step_converter::AngularStepConverter,
    pub spool_automatic_action: SpoolAutomaticAction,
    pub puller_speed_controller: puller_speed_controller::PullerSpeedController,
    pub(in ::machines::winder2) emitted_default_state: bool,
    pub(in ::machines::winder2) last_state_event: Option<api::StateEvent>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `traverse` | `ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1` |  |
| `puller` | `ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1` |  |
| `spool` | `ethercat_hal::io::stepper_velocity_el70x1::StepperVelocityEL70x1` |  |
| `tension_arm` | `tension_arm::TensionArm` |  |
| `laser` | `ethercat_hal::io::digital_output::DigitalOutput` |  |
| `traverse_controller` | `traverse_controller::TraverseController` |  |
| `traverse_end_stop` | `ethercat_hal::io::digital_input::DigitalInput` |  |
| `namespace` | `api::Winder2Namespace` |  |
| `last_measurement_emit` | `std::time::Instant` |  |
| `machine_manager` | `std::sync::Weak<smol::lock::RwLock<control_core::machines::manager::MachineManager>>` |  |
| `machine_identification_unique` | `control_core::machines::identification::MachineIdentificationUnique` |  |
| `connected_buffer` | `Option<control_core::machines::ConnectedMachine<std::sync::Weak<smol::lock::Mutex<crate::machines::buffer1::BufferV1>>>>` |  |
| `mode` | `Winder2Mode` |  |
| `spool_mode` | `SpoolMode` |  |
| `traverse_mode` | `TraverseMode` |  |
| `puller_mode` | `PullerMode` |  |
| `spool_speed_controller` | `spool_speed_controller::SpoolSpeedController` |  |
| `spool_step_converter` | `control_core::converters::angular_step_converter::AngularStepConverter` |  |
| `spool_automatic_action` | `SpoolAutomaticAction` |  |
| `puller_speed_controller` | `puller_speed_controller::PullerSpeedController` |  |
| `emitted_default_state` | `bool` | Will be initialized as false and set to true by emit_state<br>This way we can signal to the client that the first state emission is a default state |
| `last_state_event` | `Option<api::StateEvent>` |  |

##### Implementations

###### Methods

- ```rust
  pub(in ::machines::winder2) fn set_laser(self: &mut Self, value: bool) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2) fn validate_traverse_limits(inner: Length, outer: Length) -> bool { /* ... */ }
  ```
  Validates that traverse limits maintain proper constraints:

- ```rust
  pub fn traverse_set_limit_inner(self: &mut Self, limit: f64) { /* ... */ }
  ```

- ```rust
  pub fn traverse_set_limit_outer(self: &mut Self, limit: f64) { /* ... */ }
  ```

- ```rust
  pub fn traverse_set_step_size(self: &mut Self, step_size: f64) { /* ... */ }
  ```

- ```rust
  pub fn traverse_set_padding(self: &mut Self, padding: f64) { /* ... */ }
  ```

- ```rust
  pub fn traverse_goto_limit_inner(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn traverse_goto_limit_outer(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn traverse_goto_home(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn emit_live_values(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn build_state_event(self: &mut Self) -> StateEvent { /* ... */ }
  ```

- ```rust
  pub fn maybe_emit_state_event(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn emit_state(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn sync_traverse_speed(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn can_wind(self: &Self) -> bool { /* ... */ }
  ```
  Can wind capability check

- ```rust
  pub fn can_go_in(self: &Self) -> bool { /* ... */ }
  ```
  Can go to inner limit capability check

- ```rust
  pub fn can_go_out(self: &Self) -> bool { /* ... */ }
  ```
  Can go to outer limit capability check

- ```rust
  pub fn can_go_home(self: &Self) -> bool { /* ... */ }
  ```
  Can go home capability check

- ```rust
  pub(in ::machines::winder2) fn set_mode(self: &mut Self, mode: &Winder2Mode) { /* ... */ }
  ```

- ```rust
  pub(in ::machines::winder2) fn set_spool_mode(self: &mut Self, mode: &Winder2Mode) { /* ... */ }
  ```
  Apply the mode changes to the spool

- ```rust
  pub(in ::machines::winder2) fn set_traverse_mode(self: &mut Self, mode: &Winder2Mode) { /* ... */ }
  ```
  Apply the mode changes to the spool

- ```rust
  pub(in ::machines::winder2) fn set_puller_mode(self: &mut Self, mode: &Winder2Mode) { /* ... */ }
  ```
  Apply the mode changes to the puller

- ```rust
  pub(in ::machines::winder2) fn tension_arm_zero(self: &mut Self) { /* ... */ }
  ```

- ```rust
  pub fn sync_spool_speed(self: &mut Self, t: Instant) { /* ... */ }
  ```
  called by `act`

- ```rust
  pub fn set_spool_automatic_required_meters(self: &mut Self, meters: f64) { /* ... */ }
  ```

- ```rust
  pub fn set_spool_automatic_mode(self: &mut Self, mode: SpoolAutomaticActionMode) { /* ... */ }
  ```

- ```rust
  pub fn stop_or_pull_spool(self: &mut Self, now: Instant) { /* ... */ }
  ```

- ```rust
  pub fn stop_or_pull_spool_reset(self: &mut Self, now: Instant) { /* ... */ }
  ```

- ```rust
  pub fn calculate_spool_auto_progress_(self: &mut Self, now: Instant) { /* ... */ }
  ```

- ```rust
  pub fn sync_puller_speed(self: &mut Self, t: Instant) { /* ... */ }
  ```
  called by `act`

- ```rust
  pub fn puller_set_regulation(self: &mut Self, puller_regulation_mode: PullerRegulationMode) { /* ... */ }
  ```

- ```rust
  pub fn puller_set_target_speed(self: &mut Self, target_speed: f64) { /* ... */ }
  ```
  Set target speed in m/min

- ```rust
  pub fn puller_set_target_diameter(self: &mut Self, target_diameter: f64) { /* ... */ }
  ```
  Set target diameter in mm

- ```rust
  pub fn puller_set_forward(self: &mut Self, forward: bool) { /* ... */ }
  ```
  Set forward direction

- ```rust
  pub fn spool_set_regulation_mode(self: &mut Self, regulation_mode: spool_speed_controller::SpoolSpeedControllerType) { /* ... */ }
  ```

- ```rust
  pub fn spool_set_minmax_min_speed(self: &mut Self, min_speed_rpm: f64) { /* ... */ }
  ```
  Set minimum speed for minmax mode in RPM

- ```rust
  pub fn spool_set_minmax_max_speed(self: &mut Self, max_speed_rpm: f64) { /* ... */ }
  ```
  Set maximum speed for minmax mode in RPM

- ```rust
  pub fn spool_set_adaptive_tension_target(self: &mut Self, tension_target: f64) { /* ... */ }
  ```
  Set tension target for adaptive mode (0.0-1.0)

- ```rust
  pub fn spool_set_adaptive_radius_learning_rate(self: &mut Self, radius_learning_rate: f64) { /* ... */ }
  ```
  Set radius learning rate for adaptive mode

- ```rust
  pub fn spool_set_adaptive_max_speed_multiplier(self: &mut Self, max_speed_multiplier: f64) { /* ... */ }
  ```
  Set max speed multiplier for adaptive mode

- ```rust
  pub fn spool_set_adaptive_acceleration_factor(self: &mut Self, acceleration_factor: f64) { /* ... */ }
  ```
  Set acceleration factor for adaptive mode

- ```rust
  pub fn spool_set_adaptive_deacceleration_urgency_multiplier(self: &mut Self, deacceleration_urgency_multiplier: f64) { /* ... */ }
  ```
  Set deacceleration urgency multiplier for adaptive mode

- ```rust
  pub fn set_connected_buffer(self: &mut Self, machine_identification_unique: MachineIdentificationUnique) { /* ... */ }
  ```
  set connected buffer

- ```rust
  pub fn disconnect_buffer(self: &mut Self, machine_identification_unique: MachineIdentificationUnique) { /* ... */ }
  ```
  disconnect buffer

- ```rust
  pub fn reverse_connect(self: &mut Self) { /* ... */ }
  ```
  initiate connection from buffer to winder

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut std::fmt::Formatter<''_>) -> std::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Machine**
  - ```rust
    fn as_any(self: &Self) -> &dyn std::any::Any { /* ... */ }
    ```

- **MachineAct**
  - ```rust
    fn act(self: &mut Self, now: Instant) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + ''_>> { /* ... */ }
    ```

- **MachineApi**
  - ```rust
    fn api_mutate(self: &mut Self, request_body: Value) -> Result<(), anyhow::Error> { /* ... */ }
    ```

  - ```rust
    fn api_event_namespace(self: &mut Self) -> &mut Namespace { /* ... */ }
    ```

- **MachineNewTrait**
  - ```rust
    fn new<''maindevice>(params: &MachineNewParams<''_, ''_, ''_, ''_, ''_, ''_, ''_>) -> Result<Self, Error> { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToString**
  - ```rust
    fn to_string(self: &Self) -> String { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `Winder2Mode`

```rust
pub enum Winder2Mode {
    Standby,
    Hold,
    Pull,
    Wind,
}
```

##### Variants

###### `Standby`

###### `Hold`

###### `Pull`

###### `Wind`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> Winder2Mode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(mode: Mode) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &Winder2Mode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `SpoolMode`

```rust
pub enum SpoolMode {
    Standby,
    Hold,
    Wind,
}
```

##### Variants

###### `Standby`

###### `Hold`

###### `Wind`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> SpoolMode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &SpoolMode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `TraverseMode`

```rust
pub enum TraverseMode {
    Standby,
    Hold,
    Traverse,
}
```

##### Variants

###### `Standby`

###### `Hold`

###### `Traverse`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> TraverseMode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &TraverseMode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `PullerMode`

```rust
pub enum PullerMode {
    Standby,
    Hold,
    Pull,
}
```

##### Variants

###### `Standby`

###### `Hold`

###### `Pull`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PullerMode { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(mode: Winder2Mode) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **PartialEq**
  - ```rust
    fn eq(self: &Self, other: &PullerMode) -> bool { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **StructuralPartialEq**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Constants and Statics

#### Constant `VENDOR_QITECH`

```rust
pub const VENDOR_QITECH: u16 = 0x0001;
```

#### Constant `MACHINE_WINDER_V1`

```rust
pub const MACHINE_WINDER_V1: u16 = 0x0002;
```

#### Constant `MACHINE_EXTRUDER_V1`

```rust
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
```

#### Constant `MACHINE_LASER_V1`

```rust
pub const MACHINE_LASER_V1: u16 = 0x0006;
```

#### Constant `MACHINE_MOCK`

```rust
pub const MACHINE_MOCK: u16 = 0x0007;
```

#### Constant `MACHINE_BUFFER_V1`

```rust
pub const MACHINE_BUFFER_V1: u16 = 0x0008;
```

## Module `panic`

```rust
pub mod panic { /* ... */ }
```

### Types

#### Struct `PanicDetails`

```rust
pub struct PanicDetails {
    pub thread_name: Option<String>,
    pub message: Option<String>,
    pub location: Option<PanicDetailsLocation>,
    pub backtrace: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `thread_name` | `Option<String>` |  |
| `message` | `Option<String>` |  |
| `location` | `Option<PanicDetailsLocation>` |  |
| `backtrace` | `String` |  |

##### Implementations

###### Methods

- ```rust
  pub(in ::panic) fn highlight_backtrace(self: &Self) -> String { /* ... */ }
  ```
  Highlights backtrace lines that contain server or control-core crate references

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PanicDetails { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut std::fmt::Formatter<''_>) -> std::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **ToString**
  - ```rust
    fn to_string(self: &Self) -> String { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `PanicDetailsLocation`

```rust
pub struct PanicDetailsLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `file` | `String` |  |
| `line` | `u32` |  |
| `column` | `u32` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> PanicDetailsLocation { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Display**
  - ```rust
    fn fmt(self: &Self, f: &mut std::fmt::Formatter<''_>) -> std::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(location: &std::panic::Location<''_>) -> Self { /* ... */ }
    ```

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **ToString**
  - ```rust
    fn to_string(self: &Self) -> String { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Functions

#### Function `send_panic`

Thread-level panic handler for general thread crashes
Sends detailed panic information including backtrace

```rust
pub fn send_panic(thread_panic_tx: smol::channel::Sender<PanicDetails>) { /* ... */ }
```

#### Function `init_panic`

Initialize panic handling system
Sets up panic handler and starts dedicated panic monitoring thread

```rust
pub fn init_panic() -> smol::channel::Sender<PanicDetails> { /* ... */ }
```

## Module `performance_metrics`

```rust
pub mod performance_metrics { /* ... */ }
```

### Types

#### Struct `EthercatPerformanceMetrics`

Collects and manages EtherCAT performance metrics

```rust
pub struct EthercatPerformanceMetrics {
    pub(in ::performance_metrics) txrx_times: std::collections::VecDeque<std::time::Duration>,
    pub(in ::performance_metrics) loop_times: std::collections::VecDeque<std::time::Duration>,
    pub(in ::performance_metrics) last_loop_start: Option<std::time::Instant>,
    pub(in ::performance_metrics) last_log_time: std::time::Instant,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `txrx_times` | `std::collections::VecDeque<std::time::Duration>` |  |
| `loop_times` | `std::collections::VecDeque<std::time::Duration>` |  |
| `last_loop_start` | `Option<std::time::Instant>` |  |
| `last_log_time` | `std::time::Instant` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new() -> Self { /* ... */ }
  ```
  Creates a new metrics collector

- ```rust
  pub fn cycle_start(self: &mut Self) { /* ... */ }
  ```
  Records the start of a new cycle

- ```rust
  pub fn record_txrx_time(self: &mut Self, duration: Duration) { /* ... */ }
  ```
  Records the duration of a tx_rx operation

- ```rust
  pub(in ::performance_metrics) fn add_txrx_time(self: &mut Self, duration: Duration) { /* ... */ }
  ```
  Adds a tx_rx time measurement

- ```rust
  pub(in ::performance_metrics) fn add_cycle_time(self: &mut Self, duration: Duration) { /* ... */ }
  ```
  Adds a cycle time measurement

- ```rust
  pub(in ::performance_metrics) fn maybe_log_metrics(self: &mut Self) { /* ... */ }
  ```
  Logs metrics if enough time has passed

- ```rust
  pub(in ::performance_metrics) fn log_metrics(self: &Self) { /* ... */ }
  ```
  Logs the current metrics

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MetricsStats`

Statistical metrics for a set of duration measurements

```rust
pub(in ::performance_metrics) struct MetricsStats {
    pub(in ::performance_metrics) average_ms: f64,
    pub(in ::performance_metrics) percentile_9999_ms: f64,
    pub(in ::performance_metrics) stddev_ms: f64,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `average_ms` | `f64` |  |
| `percentile_9999_ms` | `f64` |  |
| `stddev_ms` | `f64` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Functions

#### Function `calculate_stats`

Calculates statistical metrics for a collection of durations

```rust
pub(in ::performance_metrics) fn calculate_stats(durations: &std::collections::VecDeque<std::time::Duration>) -> MetricsStats { /* ... */ }
```

### Constants and Statics

#### Constant `METRICS_WINDOW_SIZE`

Configuration for performance metrics collection

```rust
pub(in ::performance_metrics) const METRICS_WINDOW_SIZE: usize = _;
```

#### Constant `METRICS_LOG_INTERVAL_SECS`

```rust
pub(in ::performance_metrics) const METRICS_LOG_INTERVAL_SECS: u64 = 30;
```

## Module `rest`

```rust
pub mod rest { /* ... */ }
```

### Modules

## Module `handlers`

```rust
pub mod handlers { /* ... */ }
```

### Modules

## Module `machine_mutation`

```rust
pub mod machine_mutation { /* ... */ }
```

### Functions

#### Function `post_machine_mutate`

```rust
pub async fn post_machine_mutate(__arg0: axum::extract::State<std::sync::Arc<crate::app_state::AppState>>, __arg1: axum::Json<control_core::rest::mutation::MachineMutationBody<serde_json::Value>>) -> axum::http::Response<axum::body::Body> { /* ... */ }
```

#### Function `_post_machine_mutate`

```rust
pub(in ::rest::handlers::machine_mutation) async fn _post_machine_mutate(__arg0: axum::extract::State<std::sync::Arc<crate::app_state::AppState>>, __arg1: axum::Json<control_core::rest::mutation::MachineMutationBody<serde_json::Value>>) -> Result<(), anyhow::Error> { /* ... */ }
```

## Module `write_machine_device_identification`

```rust
pub mod write_machine_device_identification { /* ... */ }
```

### Types

#### Struct `Body`

```rust
pub struct Body {
    pub device_machine_identification: control_core::machines::identification::DeviceMachineIdentification,
    pub hardware_identification_ethercat: control_core::machines::identification::DeviceHardwareIdentificationEthercat,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `device_machine_identification` | `control_core::machines::identification::DeviceMachineIdentification` |  |
| `hardware_identification_ethercat` | `control_core::machines::identification::DeviceHardwareIdentificationEthercat` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Functions

#### Function `post_write_machine_device_identification`

```rust
pub async fn post_write_machine_device_identification(__arg0: axum::extract::State<std::sync::Arc<crate::app_state::AppState>>, __arg1: axum::Json<Body>) -> axum::http::Response<axum::body::Body> { /* ... */ }
```

## Module `init`

```rust
pub mod init { /* ... */ }
```

### Functions

#### Function `init_api`

```rust
pub fn init_api(thread_panic_tx: smol::channel::Sender<crate::panic::PanicDetails>, app_state: std::sync::Arc<crate::app_state::AppState>) -> Result<std::thread::JoinHandle<()>, anyhow::Error> { /* ... */ }
```

## Module `util`

```rust
pub mod util { /* ... */ }
```

### Types

#### Struct `ResponseUtil`

```rust
pub struct ResponseUtil {
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|

##### Implementations

###### Methods

- ```rust
  pub fn error(message: &str) -> Response<Body> { /* ... */ }
  ```

- ```rust
  pub fn ok<T: serde::Serialize>(data: T) -> Response<Body> { /* ... */ }
  ```

- ```rust
  pub fn not_found(message: &str) -> Response<Body> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `ResponseUtilError`

```rust
pub enum ResponseUtilError {
    Error(anyhow::Error),
    NotFound(anyhow::Error),
}
```

##### Variants

###### `Error`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `anyhow::Error` |  |

###### `NotFound`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `anyhow::Error` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(error: ResponseUtilError) -> Self { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `serial`

```rust
pub mod serial { /* ... */ }
```

### Modules

## Module `devices`

```rust
pub mod devices { /* ... */ }
```

### Modules

## Module `laser`

```rust
pub mod laser { /* ... */ }
```

### Types

#### Struct `Laser`

The struct of Laser Device

```rust
pub struct Laser {
    pub data: Option<LaserData>,
    pub path: String,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `data` | `Option<LaserData>` |  |
| `path` | `String` |  |

##### Implementations

###### Methods

- ```rust
  pub async fn get_diameter(self: &Self) -> Result<Length, String> { /* ... */ }
  ```

- ```rust
  pub async fn get_data(self: &Self) -> Option<LaserData> { /* ... */ }
  ```

- ```rust
  pub(in ::serial::devices::laser) async fn process(_self: Arc<RwLock<Self>>) -> Result<(), anyhow::Error> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **SerialDevice**
- **SerialDeviceNew**
  - ```rust
    fn new_serial(params: &SerialDeviceNewParams) -> Result<(DeviceIdentification, Arc<RwLock<Laser>>), anyhow::Error> { /* ... */ }
    ```

- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `LaserModbusRequsts`

```rust
pub(in ::serial::devices::laser) enum LaserModbusRequsts {
    ReadDiameter,
}
```

##### Variants

###### `ReadDiameter`

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

  - ```rust
    fn from(request: LaserModbusRequsts) -> Self { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LaserDiameterResponse`

```rust
pub(in ::serial::devices::laser) struct LaserDiameterResponse {
    pub diameter: uom::si::f64::Length,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `diameter` | `uom::si::f64::Length` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

  - ```rust
    fn try_from(value: ModbusResponse) -> Result<Self, <Self as >::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `LaserData`

```rust
pub struct LaserData {
    pub diameter: uom::si::f64::Length,
    pub last_timestamp: std::time::Instant,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `diameter` | `uom::si::f64::Length` |  |
| `last_timestamp` | `std::time::Instant` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> LaserData { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `init`

```rust
pub mod init { /* ... */ }
```

### Functions

#### Function `init_serial`

```rust
pub fn init_serial(thread_panic_tx: smol::channel::Sender<crate::panic::PanicDetails>, app_state: std::sync::Arc<crate::app_state::AppState>) -> Result<(), anyhow::Error> { /* ... */ }
```

## Module `registry`

```rust
pub mod registry { /* ... */ }
```

### Types

#### Struct `SERIAL_DEVICE_REGISTRY`

**Attributes:**

- `#[allow(missing_copy_implementations)]`
- `#[allow(non_camel_case_types)]`
- `#[allow(dead_code)]`

```rust
pub struct SERIAL_DEVICE_REGISTRY {
    pub(in ::serial::registry) __private_field: (),
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `__private_field` | `()` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **Deref**
  - ```rust
    fn deref(self: &Self) -> &SerialDeviceRegistry { /* ... */ }
    ```

- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **LazyStatic**
- **Pipe**
- **Receiver**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `socketio`

```rust
pub mod socketio { /* ... */ }
```

### Modules

## Module `init`

```rust
pub mod init { /* ... */ }
```

### Functions

#### Function `init_socketio`

```rust
pub async fn init_socketio(app_state: &std::sync::Arc<crate::app_state::AppState>) -> socketioxide::layer::SocketIoLayer { /* ... */ }
```

#### Function `handle_socket_connection`

```rust
pub(in ::socketio::init) fn handle_socket_connection(socket: socketioxide::extract::SocketRef, app_state: std::sync::Arc<crate::app_state::AppState>) { /* ... */ }
```

#### Function `setup_disconnection`

```rust
pub(in ::socketio::init) fn setup_disconnection(socket: socketioxide::extract::SocketRef, namespace_id: control_core::socketio::namespace_id::NamespaceId, app_state: std::sync::Arc<crate::app_state::AppState>) { /* ... */ }
```

#### Function `setup_connection`

```rust
pub(in ::socketio::init) fn setup_connection(socket: socketioxide::extract::SocketRef, namespace_id: control_core::socketio::namespace_id::NamespaceId, app_state: std::sync::Arc<crate::app_state::AppState>) { /* ... */ }
```

## Module `main_namespace`

```rust
pub mod main_namespace { /* ... */ }
```

### Modules

## Module `ethercat_devices_event`

```rust
pub mod ethercat_devices_event { /* ... */ }
```

### Types

#### Struct `EthercatSetupDone`

```rust
pub struct EthercatSetupDone {
    pub devices: Vec<DeviceObj>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `devices` | `Vec<DeviceObj>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> EthercatSetupDone { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `DeviceObj`

```rust
pub struct DeviceObj {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: control_core::machines::identification::DeviceIdentification,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `configured_address` | `u16` |  |
| `name` | `String` |  |
| `vendor_id` | `u32` |  |
| `product_id` | `u32` |  |
| `revision` | `u32` |  |
| `device_identification` | `control_core::machines::identification::DeviceIdentification` |  |

##### Implementations

###### Methods

- ```rust
  pub(in ::socketio::main_namespace::ethercat_devices_event) fn from_subdevice(subdevice: &SubDeviceRef<''_, SubDevicePdi<''_, PDI_LEN>>, device_identification: DeviceIdentification) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> DeviceObj { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `EthercatDevicesEvent`

```rust
pub enum EthercatDevicesEvent {
    Initializing(bool),
    Done(EthercatSetupDone),
    Error(String),
}
```

##### Variants

###### `Initializing`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `Done`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `EthercatSetupDone` |  |

###### `Error`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> EthercatDevicesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `EthercatDevicesEventBuilder`

```rust
pub struct EthercatDevicesEventBuilder();
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|

##### Implementations

###### Methods

- ```rust
  pub async fn build(self: &Self, app_state: Arc<AppState>) -> Event<EthercatDevicesEvent> { /* ... */ }
  ```

- ```rust
  pub fn initializing(self: &Self) -> Event<EthercatDevicesEvent> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `ethercat_interface_discovery_event`

```rust
pub mod ethercat_interface_discovery_event { /* ... */ }
```

### Types

#### Enum `EthercatInterfaceDiscoveryEvent`

```rust
pub enum EthercatInterfaceDiscoveryEvent {
    Discovering(bool),
    Done(String),
}
```

##### Variants

###### `Discovering`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `bool` |  |

###### `Done`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `String` |  |

##### Implementations

###### Methods

- ```rust
  pub fn build(self: &Self) -> Event<Self> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> EthercatInterfaceDiscoveryEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `machines_event`

```rust
pub mod machines_event { /* ... */ }
```

### Types

#### Struct `MachinesEvent`

```rust
pub struct MachinesEvent {
    pub machines: Vec<MachineObj>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `machines` | `Vec<MachineObj>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MachinesEvent { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MachineObj`

```rust
pub struct MachineObj {
    pub machine_identification_unique: control_core::machines::identification::MachineIdentificationUnique,
    pub error: Option<String>,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `machine_identification_unique` | `control_core::machines::identification::MachineIdentificationUnique` |  |
| `error` | `Option<String>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MachineObj { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **Debug**
  - ```rust
    fn fmt(self: &Self, f: &mut $crate::fmt::Formatter<''_>) -> $crate::fmt::Result { /* ... */ }
    ```

- **Deserialize**
  - ```rust
    fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, <__D as >::Error>
where
    __D: _serde::Deserializer<''de> { /* ... */ }
    ```

- **DeserializeOwned**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Serialize**
  - ```rust
    fn erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), Error> { /* ... */ }
    ```

  - ```rust
    fn do_erased_serialize(self: &Self, serializer: &mut dyn Serializer) -> Result<(), ErrorImpl> { /* ... */ }
    ```

  - ```rust
    fn serialize<__S>(self: &Self, __serializer: __S) -> _serde::__private::Result<<__S as >::Ok, <__S as >::Error>
where
    __S: _serde::Serializer { /* ... */ }
    ```

- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Struct `MachinesEventBuilder`

```rust
pub struct MachinesEventBuilder();
```

##### Fields

| Index | Type | Documentation |
|-------|------|---------------|

##### Implementations

###### Methods

- ```rust
  pub async fn build(self: &Self, app_state: Arc<AppState>) -> Event<MachinesEvent> { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
### Types

#### Struct `MainRoom`

```rust
pub struct MainRoom {
    pub namespace: control_core::socketio::namespace::Namespace,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `namespace` | `control_core::socketio::namespace::Namespace` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, event: MainNamespaceEvents) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
#### Enum `MainNamespaceEvents`

```rust
pub enum MainNamespaceEvents {
    MachinesEvent(control_core::socketio::event::Event<machines_event::MachinesEvent>),
    EthercatDevicesEvent(control_core::socketio::event::Event<ethercat_devices_event::EthercatDevicesEvent>),
    EthercatInterfaceDiscoveryEvent(control_core::socketio::event::Event<ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent>),
}
```

##### Variants

###### `MachinesEvent`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<machines_event::MachinesEvent>` |  |

###### `EthercatDevicesEvent`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<ethercat_devices_event::EthercatDevicesEvent>` |  |

###### `EthercatInterfaceDiscoveryEvent`

Fields:

| Index | Type | Documentation |
|-------|------|---------------|
| 0 | `control_core::socketio::event::Event<ethercat_interface_discovery_event::EthercatInterfaceDiscoveryEvent>` |  |

##### Implementations

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **CacheableEvents**
  - ```rust
    fn event_value(self: &Self) -> GenericEvent { /* ... */ }
    ```

  - ```rust
    fn event_cache_fn(self: &Self) -> CacheFn { /* ... */ }
    ```

- **Clone**
  - ```rust
    fn clone(self: &Self) -> MainNamespaceEvents { /* ... */ }
    ```

- **CloneToUninit**
  - ```rust
    unsafe fn clone_to_uninit(self: &Self, dest: *mut u8) { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **FromRef**
  - ```rust
    fn from_ref(input: &T) -> T { /* ... */ }
    ```

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **NamespaceCacheingLogic**
  - ```rust
    fn emit(self: &mut Self, event: MainNamespaceEvents) { /* ... */ }
    ```

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **ToOwned**
  - ```rust
    fn to_owned(self: &Self) -> T { /* ... */ }
    ```

  - ```rust
    fn clone_into(self: &Self, target: &mut T) { /* ... */ }
    ```

- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `namespaces`

```rust
pub mod namespaces { /* ... */ }
```

### Types

#### Struct `Namespaces`

```rust
pub struct Namespaces {
    pub main_namespace: super::main_namespace::MainRoom,
}
```

##### Fields

| Name | Type | Documentation |
|------|------|---------------|
| `main_namespace` | `super::main_namespace::MainRoom` |  |

##### Implementations

###### Methods

- ```rust
  pub fn new(socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>) -> Self { /* ... */ }
  ```

- ```rust
  pub async fn apply_mut</* synthetic */ impl FnOnce(Result<&mut Namespace, anyhow::Error>): FnOnce(Result<&mut Namespace, anyhow::Error>)>(self: &mut Self, namespace_id: NamespaceId, app_state: &Arc<app_state::AppState>, callback: impl FnOnce(Result<&mut Namespace, anyhow::Error>)) { /* ... */ }
  ```

###### Trait Implementations

- **Any**
  - ```rust
    fn type_id(self: &Self) -> TypeId { /* ... */ }
    ```

- **Borrow**
  - ```rust
    fn borrow(self: &Self) -> &T { /* ... */ }
    ```

- **BorrowMut**
  - ```rust
    fn borrow_mut(self: &mut Self) -> &mut T { /* ... */ }
    ```

- **Conv**
- **FmtForward**
- **Freeze**
- **From**
  - ```rust
    fn from(t: T) -> T { /* ... */ }
    ```
    Returns the argument unchanged.

- **HttpServerConnExec**
- **Instrument**
- **Into**
  - ```rust
    fn into(self: Self) -> U { /* ... */ }
    ```
    Calls `U::from(self)`.

- **Pipe**
- **RefUnwindSafe**
- **Same**
- **Send**
- **Sync**
- **Tap**
- **TryConv**
- **TryFrom**
  - ```rust
    fn try_from(value: U) -> Result<T, <T as TryFrom<U>>::Error> { /* ... */ }
    ```

- **TryInto**
  - ```rust
    fn try_into(self: Self) -> Result<U, <U as TryFrom<T>>::Error> { /* ... */ }
    ```

- **Unpin**
- **UnwindSafe**
- **VZip**
  - ```rust
    fn vzip(self: Self) -> V { /* ... */ }
    ```

- **WithSubscriber**
## Module `queue`

```rust
pub mod queue { /* ... */ }
```

### Functions

#### Function `send_event_with_retry`

Send a single event with retry logic

```rust
pub(in ::socketio::queue) async fn send_event_with_retry(socket: &socketioxide::extract::SocketRef, event: &std::sync::Arc<control_core::socketio::event::GenericEvent>) { /* ... */ }
```

#### Function `init_socketio_queue`

```rust
pub fn init_socketio_queue(thread_panic_tx: smol::channel::Sender<crate::panic::PanicDetails>, app_state: std::sync::Arc<crate::app_state::AppState>) { /* ... */ }
```

## Module `jemalloc_stats`

**Attributes:**

- `#[<cfg>(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]`

```rust
pub mod jemalloc_stats { /* ... */ }
```

### Functions

#### Function `init_jemalloc_stats`

```rust
pub fn init_jemalloc_stats() { /* ... */ }
```

#### Function `format_bytes`

```rust
pub(in ::jemalloc_stats) fn format_bytes(bytes: i64) -> String { /* ... */ }
```

## Functions

### Function `main`

```rust
pub(crate) fn main() { /* ... */ }
```

## Constants and Statics

### Static `GLOBAL`

**Attributes:**

- `#[<cfg>(all(not(target_env = "msvc"), not(feature = "dhat-heap")))]`

```rust
pub(crate) static GLOBAL: tikv_jemallocator::Jemalloc = Jemalloc;
```

