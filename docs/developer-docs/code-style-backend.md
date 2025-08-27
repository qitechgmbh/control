# Code Style

# General Code styling

Try to adhere to the rust style guide:
https://doc.rust-lang.org/nightly/style-guide/

# Beckhoff terminals

For Beckhoff terminals you should first implement the basic functionality, then an Interpreter of the data is built on top.
A good example of how to do this is by looking at the following code files:
el3021.rs , AnalogInputDevice

El3021 implements a so called AnalogInputDevice, which acts like an interface.
This allows us to use AnalogInputDevice instead of having to always specify which device should be used.
In Essence any Device that implements AnalogInputDevice can be used instead.

This is desired behaviour for all implemented devices, the machines should stay as modular as possible.

# Representing physical Units (Volt,Ampere,Hz,RPM etc)

To represent physical units like voltage ampere etc... we use uom (units of measurement), which acts as a wrapper for units and supplies all conversions between certain units.
For example:

```rust

```

# General Advice

- Try to avoid lifetimes if at all possible
- Try to avoid duplicate code
- Avoid excessive abstractions
- Avoid async code unless it is required
- Avoid taking Ownership of values, instead borrow the value
- When implementing a Trait for a struct, do it in the same file as the struct definition
- Split up large impl blocks into smaller impl blocks
