# Minimal Machine Boilerplate

Copy-paste template for new minimal machines on top of `qitech_lib`.
Every file has inline `// TODO:` markers showing where to edit.

---

## Quick-start checklist

### 1 — Copy this directory

```
cp -r BOILERPLATE_TEMPLATE my_new_machine
```

### 2 — Pick a unique machine ID

Open `machine_implementations/src/lib.rs` and look at the existing constants:

```rust
pub const TEST_MACHINE:              u16 = 0x0033;
pub const IP20_TEST_MACHINE:         u16 = 0x0034;
pub const ANALOG_INPUT_TEST_MACHINE: u16 = 0x0035;
// ...
pub const WAGO_750_460_MACHINE:      u16 = 0x0044;
```

Add a constant with the next free hex value:

```rust
pub const MY_MACHINE_ID: u16 = 0x00XX; // replace XX
```

### 3 — Rename throughout the template

Find & replace inside your copy of the directory:

| Find             | Replace with               |
|------------------|----------------------------|
| `MyMachine`      | `YourMachineName`          |
| `MY_MACHINE_ID`  | the constant you added     |

Module / type / constant naming:

| Concept             | Convention             | Example                          |
|---------------------|------------------------|----------------------------------|
| Machine struct      | `PascalCase`           | `Wago750_460Machine`             |
| Module directory    | `snake_case`           | `wago_750_460_machine`           |
| Machine ID constant | `SCREAMING_SNAKE_CASE` | `WAGO_750_460_MACHINE`           |
| Event enum          | `<Name>Events`         | `Wago750_460MachineEvents`       |
| Namespace struct    | `<Name>Namespace`      | `Wago750_460MachineNamespace`    |

### 4 — Fill in `mod.rs`

- Add your hardware handle fields to the struct
- Implement `get_state()` to build a `StateEvent` snapshot
- Add domain helpers (`set_output`, `turn_motor_on`, etc.) that mutate
  hardware then call `self.emit_state()`

### 5 — Fill in `new.rs`

- Pick **Pattern A** (Beckhoff terminal) or **Pattern B** (WAGO coupler +
  module) — delete the unused block
- Uncomment & adapt the imports + acquisition code
- Populate the `Self { … }` constructor with your hardware fields

### 6 — Fill in `api.rs`

- Add fields to `StateEvent` for everything the UI displays
- Add `Mutation` variants for every UI-triggered action
- Add dispatch arms in `api_mutate`
- For a read-only machine, leave `Mutation` empty and short-circuit
  `api_mutate` to `Ok(())`

### 7 — Fill in `act.rs`

- If the machine needs to read hardware every cycle before emitting,
  do that inside `act()` before `emit_state()`
- Most machines leave `react()` empty

### 8 — Register the module

`machine_implementations/src/minimal_machines/mod.rs`:

```rust
pub mod my_new_machine;
```

### 9 — Register in the machine registry

`machine_implementations/src/registry.rs`:

```rust
// import alongside the other minimal machines:
use crate::minimal_machines::my_new_machine::YourMachineName;

// inside lazy_static! MACHINE_REGISTRY:
mc.register::<YourMachineName>(vec![YourMachineName::MACHINE_IDENTIFICATION]);
```

### 10 — Add the slug

`machine_implementations/src/machine_identification.rs` — **required**, missing
this triggers a runtime panic ("Unknown machine id") on first instantiation.

```rust
// near the bottom, alongside the other `use crate::…;`:
use crate::MY_MACHINE_ID;

// inside the `slug()` match:
x if x == MY_MACHINE_ID => "my_new_machine".to_string(),
```

The slug must equal the module directory name.

### 11 — Verify it compiles

```
cargo check -p machine_implementations
```

---

## File overview

| File      | Responsibility                                                  |
|-----------|-----------------------------------------------------------------|
| `mod.rs`  | Struct, `MACHINE_IDENTIFICATION`, `get_state`/`emit_state`, helpers |
| `new.rs`  | `MachineNew::new` — hardware init, runs once at startup         |
| `act.rs`  | `Machine::act` / `react` / `get_identification` — control loop  |
| `api.rs`  | `StateEvent`, `Mutation`, `Namespace`, `MachineApi` impl        |

## Architecture diagram

```
control cycle (~1 kHz)
      │
      ▼
  Machine::act() ── drains receiver ──► MachineApi::act_machine_message()
      │                                          │
      │                                    SubscribeNamespace   → emit_state()
      │                                    UnsubscribeNamespace
      │                                    HttpApiJsonRequest   → api_mutate()
      │                                    RequestValues        → serialize state
      │
      └── every ~33 ms ──► emit_state() ──► socket.io subscribers
```

## Hardware patterns

### Pattern A — Beckhoff EtherCAT terminal

```
EtherCAT bus:  [EK1100 / coupler (role 0)] [EL2004 (role 1)] [EL3021 (role 2)] …
```

Role indices match `device_roles` in the machine's frontend `properties.ts`.
Acquire with:

```rust
let el2004: Rc<RefCell<EL2004>> =
    hw.try_get_ethercat_device_by_role(1)?;
```

If you need the EtherCAT subdevice address (for CoE writes), use
`try_get_ethercat_device_and_addr_by_role::<T>` instead.

### Pattern B — WAGO 750 coupler + expansion modules

```
EtherCAT bus:  [WAGO 750-354 (role 0)] ── local backplane ──
                                          [module slot 0]
                                          [module slot 1]
                                          …
```

The coupler is **always** role 0. The expansion modules are discovered by
calling `Wago750_354::initialize_modules` then `init_slot_modules`, and are
taken out of `coupler.slot_devices[i]` by slot index (0-based) and downcast
with `downcast_subdevice::<T>(dev)?`.

WAGO module subdevices are owned by the machine (`Box<T>`), not shared
through `Rc<RefCell<…>>` — once taken from the coupler slot they live on
the machine struct.

## Reference machines (already ported)

Use these as concrete examples while filling in the template:

| Hardware pattern                | Reference                       |
|---------------------------------|---------------------------------|
| Beckhoff DI + DO terminals      | `digital_input_test_machine`    |
| Beckhoff analog input (EL3021)  | `analog_input_test_machine`     |
| Beckhoff stepper (EL7031_0030)  | `motor_test_machine`            |
| WAGO 750-354 + 750-460 (4× RTD) | `wago_750_460_machine`          |
| WAGO 750-354 + 750-1506 (8× DIO)| `wago_8ch_dio_test_machine`     |
| Plain stepper machine           | `test_machine_stepper`          |

## Key traits

- `MachineNew` (this crate) — `new(hw: MachineHardware) -> Result<Self>`
- `Machine` (`qitech_lib::machines`) — `act`, `react`, `get_identification`
- `MachineApi` (this crate) — `act_machine_message`, `get_api_sender`,
  `api_mutate`, `api_event_namespace`
- `QiTechMachine` (this crate) — marker trait, automatically satisfied once
  the three above are implemented
