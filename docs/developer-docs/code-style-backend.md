# Code Style

# General Code styling

Try to adhere to the rust style guide:
https://doc.rust-lang.org/nightly/style-guide/

# Beckhoff terminals and Machines

For Beckhoff terminals you should first implement the basic functionality, then an Interpreter of the device/data is built on top.
A good example of how to do this is by looking at the following code:
el3021.rs , AnalogInputDevice

El3021 implements a so called AnalogInputDevice, which acts like an interface.
This allows us to use AnalogInputDevice instead of having to always specify which device should be used.
In Essence any Device that implements AnalogInputDevice can be used.

AnalogInputDevice is then used for creating an AnalogInput, which is plug and play for machines.

Then a further layer of Abstraction is used on top,
where the data is interpreted and incorporated into a sort of control loop for certain functions, for example:

```rust
// ScrewSpeedController controlls the speed (frequency) and direction of the motor
// AnalogInput is used to collect measurements from the pressure sensor.
// screw_speed_controller.rs
#[derive(Debug)]
pub struct ScrewSpeedController {
    pub pid: ClampingTimeagnosticPidController,
    pub target_pressure: Pressure,
    pub target_rpm: AngularVelocity,
    pub inverter: MitsubishiCS80,
    pressure_sensor: AnalogInput,
    last_update: Instant,
    uses_rpm: bool,
    forward_rotation: bool,
    transmission_converter: TransmissionConverter,
    frequency: Frequency,
    maximum_frequency: Frequency,
    minimum_frequency: Frequency,
    motor_on: bool,
    nozzle_pressure_limit: Pressure,
    nozzle_pressure_limit_enabled: bool,
}

// Under the hood for ExtruderV2 a El3021 is used:
// (from new.rs of extruder)
let pressure_sensor = AnalogInput::new(el3021, EL3021Port::AI1);

// But the extruder does not directly use the AnalogInput, instead the logic is decoupled in the screw_speed_controller:
let mut extruder: ExtruderV2 = Self {
    namespace: ExtruderV2Namespace::new(params.socket_queue_tx.clone()),
    last_measurement_emit: Instant::now(),
    mode: ExtruderV2Mode::Standby,
    temperature_controller_front: temperature_controller_front,
    temperature_controller_middle: temperature_controller_middle,
    temperature_controller_back: temperature_controller_back,
    temperature_controller_nozzle: temperature_controller_nozzle,
    screw_speed_controller: screw_speed_controller,
    emitted_default_state: false,
};

// Then in act.rs
// The extruder simply calls the update function of the screw_speed_controller,
// which contains "all" of the logic needed to properly drive the motor
self.screw_speed_controller.update(now_ts, true);


```

This is desired behaviour for all implemented devices, the machines should stay as modular as possible.

# Representing physical Units (Volt,Ampere,Hz,RPM etc)

To represent physical units like voltage ampere etc... we use uom (units of measurement), which acts as a wrapper for units and supplies all conversions between certain units.
For example:

```rust
// Lets say a device measures voltage, you just got the raw value
let volts : ElectricPotential = ElectricPotential::new::<volt>(raw_value_here);
// Then you can do this to get milli_volts
// In api responses we always use the RAW float value (64bit):
let milli_volts : f64 = volts.get::<millivolt>();
```

# General Advice

- Try to avoid duplicate code (DRY principle)
- Try to avoid lifetimes if at all possible
- Avoid excessive abstractions
- Avoid async code unless it is required
- Avoid taking Ownership of values, instead borrow the value
- When implementing a Trait for a struct, do it in the same file as the struct definition
- Split up large impl blocks into smaller impl blocks
