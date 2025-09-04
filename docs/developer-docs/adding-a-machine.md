# How can I add a new Machine?

Adding a new machine follows a common structure in `server/machines/`.



# TLDR

Summary

When adding a new machine:

- Create Folder server/machines/YOUR_MACHINE_NAME/.
- Add the 4 files (mod.rs, new.rs, act.rs, api.rs).
- Register it in machines/mod.rs with a unique constant.
- mod.rs: define the struct + emit functions.
- new.rs: assemble devices/controllers into the machine.
- act.rs: implement the machine loop (MachineAct).
- api.rs: define request (mutations) and response (events) types.

Tip: Start simple (copy MockMachine) and grow complexity as needed (add EtherCAT devices, controllers, etc.).

---

## 1. Create the folder

Create a new directory:
server/machines/YOUR_MACHINE_NAME

At a minimum, this folder needs the following files:

- `mod.rs`
- `new.rs`
- `act.rs`
- `api.rs`

---

## 2. Register the module

Add your machine to `server/machines/mod.rs`:

```rust
// mod.rs
pub mod buffer1;
pub mod extruder1;
pub mod laser;
pub mod mock;
pub mod registry;
pub mod winder2;
pub mod YOUR_MACHINE;

pub const VENDOR_QITECH: u16 = 0x0001;
pub const MACHINE_WINDER_V1: u16 = 0x0002;
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
pub const MACHINE_LASER_V1: u16 = 0x0006;
pub const MACHINE_MOCK: u16 = 0x0007;
pub const MACHINE_BUFFER_V1: u16 = 0x0008;
pub const YOUR_MACHINE_V1: u16 = 0xffff; // pick a unique ID to avoid collisions also DO NOT touch any already defined ids!!
```

## 3. mod.rs

This file defines your machine struct and the required event emitters.

Look at MockMachine’s mod.rs for the simplest possible version.

At a minimum, you need:
- emit_state
- emit_live_values
- A machine struct for state(e.g. MockMachine, Extruder, Laser, Winder, …)

More complex machines (e.g. Extruder, Winder) encapsulate controllers that in turn manage devices like EtherCAT terminals.

## 4 new.rs

This file implements MachineNewTrait for your machine.
Here you assemble your machine from its parts — devices, controllers, and any state.

Minimal example: see MockMachine → basically no devices, just returns a struct.

Medium complexity: see Laser → wraps a single serial USB device.

High complexity: see Extruder → configures multiple EtherCAT devices inside controllers.

Think of new.rs as the factory:
This is the place to declare which devices make up your machine and how they’re configured.

## 5. act.rs

This file implements the machine control loop using MachineAct.
The act() method is called repeatedly by the control runtime.

```rs
impl MachineAct for YourMachine {
    fn act(&mut self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let now = Instant::now();

            // Update state, e.g. step a controller
            self.some_controller.update();

            // Emit live values ~30 Hz if machine is in Running mode
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0) {
                self.maybe_emit_state_event();
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
```

## 6. api.rs

Defines the frontend-facing API for your machine:

Mutations → requests that the machine accepts
(e.g. start/stop commands, setpoints, configuration updates).

Events → responses emitted to the frontend
(e.g. live sensor data, alarms, machine state changes).

Look at MockMachine’s api.rs for a minimal version.

```rs
/// Mutations are API requests to control the machine
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum YourMachineMutation {
    Start,
    Stop,
    SetSpeed { rpm: f64 },
}

/// Events are responses sent to the frontend
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum YourMachineEvent {
    StateChanged { state: String },
    LiveValues { rpm: f64, torque: f64 },
}

```

