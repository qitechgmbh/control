# How can i add a new Machine?

1. First off create the folder server/machines/YOUR_MACHINE_NAME
2. at a minimum you need to create an act.rs, api.rs,mod.rs and new.rs file
3. add the module in machines/mod.rs
4. and choose an unused identification for the Machine

mod.rs should look like this:

```rust
// mod.rs
pub mod buffer1;
pub mod extruder1;
pub mod laser;
pub mod mock;
pub mod registry;
pub mod winder2;
pub mod YOUR_MACHINE

pub const VENDOR_QITECH: u16 = 0x0001;
pub const MACHINE_WINDER_V1: u16 = 0x0002;
pub const MACHINE_EXTRUDER_V1: u16 = 0x0004;
pub const MACHINE_LASER_V1: u16 = 0x0006;
pub const MACHINE_MOCK: u16 = 0x0007;
pub const MACHINE_BUFFER_V1: u16 = 0x0008;
pub const YOUR_MACHINE_V1: u16 = 0xffff;
```

# mod.rs

Look at MockMachine's mod.rs, this shows you what you need at a minimum:
emit_state, maybe_emit_state, emit_live_values functions, MockMachine ...

Here MockMachine is the Machine state struct if you will.
In more complex machines like extruder or winder there would be a lot of controllers encapsulating all kinds of devices like ethercat devices for example.

# new.rs

In the new.rs file the impl MachineNewTrait has to be implemented for your Machine struct.
in new() you configure all your devices that are part of the machine.

Look at Laser for a Machine that is essentially just a Serial Usb Device or Extruder for a more complex machine using multiple ethercat devices encapsulated in controllers.

The absolutely minimal example would be the MockMachine's new.rs, which basically doesnt even use any devices.

# act.rs

```rust
// Every machine needs to implement MachineAct
// the act method is called by the controlling loop
// you need to wrap machine logic, so that it works inside a loop contiuously executed
impl MachineAct for YOUR_MACHINE {
    fn act(&mut self, _now_ts: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let now = Instant::now();

            // update some state on YOUR_MACHINE struct
            self.some_controller.update();

            // Only emit live values if machine is in Running mode
            // The live values are updated approximately 30 times per second
            if now.duration_since(self.last_measurement_emit) > Duration::from_secs_f64(1.0 / 30.0)
            {
                // checks if state has changed, if yes -> send response
                self.maybe_emit_state_event();
                self.emit_live_values();
                self.last_measurement_emit = now;
            }
        })
    }
}
```

# api.rs

Look at MockMachines api.rs.
Mutations are request formats accepted by our machine's api.
While Events are responses sent to the frontend.
