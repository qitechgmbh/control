# Extending

New machine types are added in `machine_implementations`. A machine is a small, self-contained module that describes its hardware, the state it shows the operator, and the commands it accepts. Once registered, the runtime discovers it, builds it when its hardware appears on the bus, and serves it over the API. A boilerplate template under `machine_implementations/src/minimal_machines/` shows the file layout to copy, with inline `// TODO:` comments marking every place that needs editing.

## Anatomy of a machine

A machine module is split into a few files, each with a clear job:

- **`mod.rs`** — defines the machine struct (its hardware handles and its state) and the domain logic that operates on it.
- **`new.rs`** — builds the machine from the hardware that was detected at startup. This runs once, when the machine is created.
- **`api.rs`** — declares the state the machine broadcasts to the operator interface and the commands the interface can send, and wires those commands to the machine's logic.
- **`act.rs`** — the per-cycle update. It is called every control cycle, applies any pending commands, and publishes the machine's current state to subscribers at the UI refresh rate.

## Identification and registration

Every machine is addressed by a **machine identification**: a vendor id, a machine id, and a serial number. Wiring a new machine in means three things:

1. **Add a machine-id constant** for the new type.
2. **Register the machine type** in the central machine registry, bound to its identification. The registry is what lets the runtime build the correct machine when that identification is discovered on the bus.
3. **Add a slug** — a short, stable name for the machine id. The slug is what the REST and Socket.IO routes use: a machine is reached at `/machine/{vendor}/{machine}/{serial}`.

Registering the module and giving the id a slug are both required; a missing slug causes a runtime error the first time the machine is built.

## The frontend half

Every backend machine has a matching frontend module that mirrors its state and renders its screen. Copy the boilerplate frontend module, re-declare the machine's state as Zod schemas, and build its page. See **Frontend**.

## Build check

When the module is wired up, confirm it compiles:

```bash
cargo check -p machine_implementations
```
