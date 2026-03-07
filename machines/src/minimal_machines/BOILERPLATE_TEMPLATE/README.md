# Minimal Machine Boilerplate

This directory is a copy-paste template for creating a new minimal machine.
Follow the steps below in order. Each file has inline `// TODO:` comments that
mark every place you need to edit.

---

## Quick-start checklist

### 1 — Copy this directory

```
cp -r BOILERPLATE_TEMPLATE my_new_machine
```

### 2 — Pick a unique machine ID

Open `machines/src/lib.rs` and look at the existing constants:

```rust
pub const TEST_MACHINE:            u16 = 0x0033;
pub const IP20_TEST_MACHINE:       u16 = 0x0034;
pub const ANALOG_INPUT_TEST_MACHINE: u16 = 0x0035;
// ...
```

Add a new constant with the next available hex value:

```rust
pub const MY_MACHINE_ID: u16 = 0x00XX; // replace XX
```

### 3 — Rename throughout the template

Do a project-wide find & replace inside your new directory:

| Find          | Replace with              |
|---------------|---------------------------|
| `MyMachine`   | `YourMachineName`         |
| `MY_MACHINE_ID` | the constant you added  |

### 4 — Fill in `mod.rs`

- Add hardware fields to the struct (e.g. `douts: [DigitalOutput; 4]`)
- Add domain-specific helper methods (`set_output`, `read_inputs`, …)

### 5 — Fill in `new.rs`

- Choose Pattern A (Beckhoff terminal) or Pattern B (WAGO coupler + module)
- Uncomment and adapt the hardware imports and initialization code
- Add your hardware fields to the `Self { … }` constructor at the bottom

### 6 — Fill in `api.rs`

- Add fields to `StateEvent` that the UI needs to display
- Add `Mutation` variants for every action the UI can trigger
- Implement `api_mutate` to dispatch each mutation to your helper methods

### 7 — Fill in `act.rs`

- If your machine reads hardware on every cycle, add that call inside `act()`
  before `emit_state()` is called
- Everything else is boilerplate and usually does not need changes

### 8 — Register the module in `minimal_machines/mod.rs`

```rust
pub mod my_new_machine;
```

### 9 — Register the machine in `machines/src/registry.rs`

```rust
// at the top of registry.rs — add the import:
use crate::minimal_machines::my_new_machine::YourMachineName;

// inside the lazy_static! MACHINE_REGISTRY block — add the registration:
mc.register::<YourMachineName>(YourMachineName::MACHINE_IDENTIFICATION);
```

### 10 — Verify it compiles

```
cargo check -p machines
```

---

## File overview

| File      | Responsibility                                              |
|-----------|-------------------------------------------------------------|
| `mod.rs`  | Struct definition, `Machine` trait, business logic helpers  |
| `new.rs`  | `MachineNewTrait` — hardware init, called once at startup   |
| `api.rs`  | Events, mutations, `MachineApi` trait                       |
| `act.rs`  | `MachineAct` trait — update loop called every EtherCAT cycle|

## Architecture diagram

```
EtherCAT cycle
      │
      ▼
  act() ──── drains message queue ──► act_machine_message()
      │                                    │
      │                              SubscribeNamespace → emit_state()
      │                              HttpApiJsonRequest → api_mutate()
      │                              RequestValues      → serialize state
      │
      └── every 33 ms ──► emit_state() ──► socket.io subscribers
```

## Hardware patterns

### Pattern A — Beckhoff EtherCAT terminal

```
EtherCAT bus:  [Bus Coupler (role 0)] [EL2004 (role 1)]
```

The role index matches the `device_roles` array in `properties.ts`.

### Pattern B — WAGO 750 coupler + expansion modules

```
EtherCAT bus:  [WAGO 750-354 (role 0)] ─── local backplane ───
                                            [750-530 slot 0]
                                            [750-402 slot 1]
                                            ...
```

The coupler is always role 0. Modules are found by slot index (0-based)
after calling `initialize_modules` + `init_slot_modules`.

## Naming conventions

| Concept            | Convention                     | Example                       |
|--------------------|-------------------------------|-------------------------------|
| Machine struct     | `PascalCase`                   | `WagoDiMachine`               |
| Module directory   | `snake_case`                   | `wago_di_machine`             |
| Machine ID constant| `SCREAMING_SNAKE_CASE`         | `WAGO_DI_MACHINE`             |
| Event struct       | `<Name>Events` enum            | `WagoDiMachineEvents`         |
| Namespace struct   | `<Name>Namespace`              | `WagoDiMachineNamespace`      |
