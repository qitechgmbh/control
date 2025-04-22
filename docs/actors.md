# Actors

Every cycle in the control loop all "actors" are called. Actors are structs that implement the `Actor` trait which exposes the `act` function.
In the `act` function the actor can execute code like reading or writing to IOs.

In this example the actor will togge the output of a digital output every cycle.
```rust
#[derive(Debug)]
pub struct DigitalOutputToggler {
    output: DigitalOutput,
}

impl Actor for DigitalOutputToggler {
    fn act(&mut self, _now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
    Box::pin(async move {
        {
            let state = (self.output.state)().await;
            match state.output.into() {
                true => (self.output.write)(false.into()).await,
                false => (self.output.write)(true.into()).await,
            }
        }
    }
}

```

Actors can be nested. A common example for this are machines that have multiple actors. In this case the machine has to call the sub-actors in its `act` function.
```rust
struct Machine1 {
    toggler1: DigitalOutputToggler,
    toggler2: DigitalOutputToggler,
    // ...
}

impl Actor for Machine1 {
    fn act(&mut self, now: Instant) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.toggler1.act(now).await;
            self.toggler2.act(now).await;
        })
    }
}
```

The `act` function should be as fast as possible, because delays slow down the control loop.