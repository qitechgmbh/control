# Step by Step instructions for porting machines to the new api

## 1. Fixing Import Errors

`control_core` has been merged into `qitech_control`. Therefore, all imports referencing `control_core` are now invalid.

The affected modules now live directly in the crate root. Simply replacing `control_core::*` with `crate::*` should resolve the issue.

Example:

Before:

```rust
use control_core::converters::angular_step_converter::AngularStepConverter;
use control_core::helpers::interpolation::scale;
```

After:
```rust
use crate::converters::angular_step_converter::AngularStepConverter;
use crate::utils::interpolation::scale;
```

## 2. api.rs

### 2.1. Removing SocketIO related Boilerplate

SocketIO and networking have been moved out of machines and thus many boilerplate related
to such things should be removed.

Example of things that are redundant now:

Serde derives:
```rust
#[derive(Serialize, Deserialize)] // < remove these derives
pub enum Mode {
    ...
}
```

Events:
```rust
pub enum MyMachineEvents {
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}
```

Namespace:

```rust
#[derive(Debug)]
pub struct MyMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<MyMachineEvents> for MyMachineNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: MyMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        match &mut self.namespace {
            Some(ns) => ns.emit(event, &buffer_fn),
            None => (),
        }
    }
}

impl CacheableEvents<Self> for MyMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
        }
    }
}
```

MachineApi trait impl:

```rust
impl MachineApi for Winder2 {
    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    /// IMPORTANT NOTE: Keep a copy of this function for later migration !!!
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        ...
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            ...
        }
    }
}
```

### 2.2. Reusing LiveValuesEvent as Migration target for Measurements

The new resource api exposes properties which have getters and setters and should ideally
be used directly in place where they're used, however that introduces changes everwhere
they're used which leads to more things to keep track of. Instead we will be reusing the old
struct as export target for measurements for a faster and less breaking migration.

Before:

```rust
#[derive(Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub traverse_position: Option<f64>,
    pub puller_speed: f64,
    pub spool_rpm: f64,
    pub tension_arm_angle: f64,
    pub spool_progress: f64,
}
```

After:

```rust
use qitech_lib::units::{Angle, AngularVelocity, Length, Velocity};
use qitech_framework::machine::Measurement;

#[derive(Debug, Clone, Default)]
pub struct Measurements {
    pub traverse_position: Measurement<Option<Length>>,
    pub puller_speed: Measurement<Velocity>,
    pub spool_rpm: Measurement<AngularVelocity>,
    pub tension_arm_angle: Measurement<Angle>,
    pub spool_progress: Measurement<Length>,
}
```

### 2.3. Reusing StateEvent as Migration target for StateProperties

Just like measurements, we don't manually export them but have state properties for exposing
state.

Before:
```rust
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StateEvent {
    pub higher_tolerance: f64,
    pub lower_tolerance: f64,
    pub target_diameter: f64,
    pub in_tolerance: bool,
}
```

After:
```rust
#[derive(Debug, Clone, Default)]
pub struct StateProperties {
    pub higher_tolerance: StateProperty<Length>,
    pub lower_tolerance: StateProperty<Length>,
    pub target_diameter: StateProperty<Length>,
    pub in_tolerance: StateProperty<bool>,
}
```

### 2.4. Isolating Mutations into ConfigProperties and Commands

Currently the common way to declare api requests is using one large enum with all requests
and parse that json. However machines do not understand json anymore, in fact machines 
aren't responsible for handling api requests at all, those are all handled by the runtime.
However in order for the runtime to expose configuration and behaviour we need to first
differentiate between a configuration and an action or command. A configuration defines
`how` a machine should do something a command defines what a machine should do. 
The mutation enum currently mixes both and thus they first need to be filtered out.

Example for what should become a configuration:
```rust
SetTraverseLimitOuter(f64),
SetTraverseLimitInner(f64),
SetTraverseStepSize(f64),
SetTraversePadding(f64),
```

These do not change what the machine does, but how therefore these are configurations.

Example for what should become a command:
```rust
GotoTraverseLimitOuter,
GotoTraverseLimitInner,
GotoTraverseHome,
```

These are direct instructions to the machine to execute and action, therefore these are
commands.

```rust
SetMode(Mode), // < mode for winder, instructs the winder what to do
```

This looks like a configuration, *however*, this actually instructs the machine to go into
and operating mode and thus is an instruction to *do* something, therefore this is a command.


### 2.5 Creating Commands

Now that've established what a command is and what it is not, we can start extracting 
commands out of the Mutation.

As example lets take these 3 mutations and extract them into commands 

Before:

```rust
pub enum Mutation {
    GotoTraverseLimitInner,
    GotoTraverseHome,
    SetMode(Mode),
    // ... 
}

impl MachineApi for Winder2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetMode(mode) => self.set_mode(&mode.into()),
            Mutation::GotoTraverseLimitInner => self.traverse_goto_limit_inner(),
            Mutation::GotoTraverseHome => self.traverse_goto_home(),
            // ....
        }

        Ok(())
    }

    // ....
}
```

After:

```rust
impl Winder2 {
    pub fn cmd_traverse_goto_home(&mut self) -> CommandExecuteResult {
        self.traverse_goto_home();
        Ok(())
    }

    pub fn cmd_traverse_goto_limit_outer(&mut self) -> CommandExecuteResult {
        self.traverse_goto_limit_outer();
        Ok(())
    }

    pub fn cmd_traverse_goto_limit_inner(&mut self) -> CommandExecuteResult {
        self.traverse_goto_limit_inner();
        Ok(())
    }

    pub fn cmd_enter_standby_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Standby);
        Ok(())
    }

    pub fn cmd_enter_hold_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Hold);
        Ok(())
    }

    pub fn cmd_enter_pull_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Pull);
        Ok(())
    }

    pub fn cmd_enter_wind_mode(&mut self) -> CommandExecuteResult {
        self.set_mode(&Winder2Mode::Wind);
        Ok(())
    }
}
```

A command is always declared with the machine itself as &mut and no other arguments and must
always return Result<(), String> or CommandExecuteResult as alias.

Because commands don't have input arguments the SetMode has been split into 4 individual 
functions.

The cmd_* prefix is used here to avoid name clashes since this machine already has functions
with matching names. Since our goal is to avoid changing code wherever possible we create
these wrappers for now. You may wish to later collapse or merge them *after* the initial 
migration is completed. 


### 2.6 Defining Config Properties

Now onto the last category we need to migrate, config properties.
Let't take the example below:

```rust
pub enum Mutation {
    SetTraverseLimitOuter(f64), // is mm
    SetTraverseLimitInner(f64), // is mm
    SetTraverseStepSize(f64), // is mm
    SetTraversePadding(f64), // is mm
}
```

We simply create a new struct named ConfigProperties
and put all the remaining mutations in as config properties.


```rust
#[derive(Debug, Clone, Default)]
pub struct ConfigProperties {
    pub traverse_limit_inner: ConfigProperty<Length>,
    pub traverse_limit_outer: ConfigProperty<Length>,
    pub traverse_step_size: ConfigProperty<Length>,
    pub traverse_padding: ConfigProperty<Length>,
}
```

Don't worry about the old setter functions used to set them for now.
We will go them later. For now simply create the struct with the appropiate type.

## 3. Defining the Schema

Now that've isolated all resources and have a clear overview of all of them we can now
define the schema or interface of our machine so other systems can understand our interface.
For that create a new yaml file ideally in a /schemas directory for better organization and
name it your machines name in snake case. For this example we will be using my_machine.yaml.
Let's use all the resources we've defined above and put them into the schema:

```yaml
config:
  traverse:
    limit:
      inner: !millimeter
        default: 22.0

      outer: !millimeter
        default: 22.0

    step_size: !millimeter
      default: 1.75

    padding: !millimeter
      default: 0.88

  puller:
      gear_ratio: !enum
        variants: [one_to_one, one_to_five, one_to_ten]
        default: one_to_one

state:
  is_homed: !boolean
  is_going_in: !boolean
  is_going_out: !boolean

measurements:
  traverse:
    position: !length
      nullable: true

  puller:
    speed: !meter_per_minute

  spool:
    rpm: !revolution_per_minute
    progress: !meter

  tension_arm:
    angle: !degree

commands:
  enter_standby_mode: !command
  enter_hold_mode: !command
  enter_pull_mode: !command
  enter_wind_mode: !command

  traverse:
    goto_home: !command
    goto_limit_inner: !command
    goto_limit_outer: !command
```

Config properties as seen all require a default value, whereas measurements and 
state properties do not. All 3 of them support nullable as option. Enums require
a variant field that contains all values.


## 4. mod.rs

### 4.1 Refactoring Machine type fields

Now that we have the resources available we can put them into our machine while removing
leftovers from the old system.


Old fields that can be removed:

```rust
pub struct Winder2 {
    api_receiver: Receiver<MachineMessage>,
    api_sender: Sender<MachineMessage>,

    namespace: Winder2Namespace,
    last_measurement_emit: Instant,
    pub machine_identification_unique: MachineIdentificationUnique,

    emitted_default_state: bool,

    // ...
}
```

New fields to add:

```rust
pub struct Winder2 {
    // ...

    config_props: ConfigProperties,
    state_props: StateProperties,
    measurements: Measurements,
}
```

Now in addition you can remove any mentions of the now removed fields.

### 4.2 Updating the properties

replace 

```rust
    pub fn get_live_values(&self) -> LiveValuesEvent {
        let angle_deg = self.tension_arm.get_angle().unwrap();

        // ...

        LiveValuesEvent {
            tension_arm_angle: angle_deg,
        }
    }
```

with 

```rust
    fn update_measurements(&mut self) {
        let angle_deg = self.tension_arm.get_angle().unwrap();

        // ...

        self.measurements.tension_arm_angle.set(angle_deg);
    }
```

same applies for the previous state event

replace 

```rust
pub fn build_state_event(&mut self) -> StateEvent {
    StateEvent {
        mode: self.mode
        // ...
    }
}
```

with 

```rust
pub fn update_states(&mut self) {
    self.states.mode.set(self.mode);
}
```

### 4.3 Removing emit_live_values and emit_state

Simply remove every mention of `self.emit_state();` and `self.emit_live_values();`

## 5. new.rs

Now it's time register all our resources so we can start using them.

### 5.1 Implementing MachineBuild for our machine

MachineNew has been replaced by MachineBuild which looks like this:

```rust
use qitech_framework::machine::MachineBuild;
use qitech_framework::machine::BuildContext;
use qitech_framework::machine::error::BuildResult;

impl MachineBuild for Winder2 {
    fn build(ctx: BuildContext) -> BuildResult<Self> {
        // ...
    }
}
```

ctx will be used to retrieve hardware and register resources.


#### 5.1.1 Hardware

Old signatures like this: 

```rust
fn new(hw: MachineHardware) -> Result<Self, Error> {
    let _ek1100 = hw.try_get_ethercat_device_and_addr_by_role::<EK1100>(0)?;
    let el2002 = hw.try_get_ethercat_device_and_addr_by_role::<EL2002>(1)?;
    let el7041 = hw.try_get_ethercat_device_and_addr_by_role::<EL7041_0052>(2)?;
    let el7031 = hw.try_get_ethercat_device_and_addr_by_role::<EL7031>(3)?;
    let el7031_0030 = hw.try_get_ethercat_device_and_addr_by_role::<EL7031_0030>(4)?;

    // ...

    let interface: EtherCATThreadChannel = match &hw.ethercat_interface {
        Some(ecat_interface) => ecat_interface.clone(),
        None => {
            return Err(anyhow::anyhow!(
                "Winder2: No EtherCat Interface was supplied!"
            ));
        }
    };

    // ...
}
```

Translate to this signature:

```rust
fn build(ctx: BuildContext) -> BuildResult<Self> {
    let _ek1100 = ctx.find_ethercat_device_and_addr::<EK1100>(0)?;
    let el2002 = ctx.find_ethercat_device_and_addr::<EL2002>(1)?;
    let el7031_0030_spool = ctx.find_ethercat_device_and_addr::<EL7031_0030>(2)?;
    let el7031 = ctx.find_ethercat_device_and_addr::<EL7031>(3)?;
    let el7031_0030 = ctx.find_ethercat_device_and_addr::<EL7031_0030>(4)?;
    // ...

    let interface = ctx.get_ethercat_interface()?;
    // ...
}
```

as you can see they are really similar when it comes to loading hardware.


#### 5.1.2 Resources

Now finally it is time to register our resources. 

##### Registerig A Config Property

Lets register some config properties from our schema above:

```yaml
config:
  traverse:
    limit:
      inner: !millimeter
        default: 22.0

      outer: !millimeter
        default: 22.0
```

Remember when I said to remember the old setters used by the mutations. 
Now we will need them again so changes in a config property are reflected at the data origin.

inside of `api.rs` for each property we defined in ConfigProperties define a function
with the signature `func(&mut MyMachine) -> Result<(), String>` which calls the same 
function the previous mutation did. Example:

```rust
pub struct ConfigProperties {
    pub traverse_limit_inner: ConfigProperty<Length>,
    // ...
}

impl WinderV2 {
    pub fn on_traverse_limit_inner_changed(&mut self) -> Result<(), String> {
        self.traverse_set_limit_inner(
            self.config_props.traverse_limit_inner.get_as::<millimeter>()
        )
    }
}
```

where the old signature was:

```rust
impl MachineApi for Winder2 {
    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetTraverseLimitInner(limit) => self.traverse_set_limit_inner(limit),
            // ...
        }
    }

    // ...
}
```

Using this we can now register our config property with a callback to also update the
source value when the config property is modified. Again later you may choose to replace 
the source value with the property itself but for now use wrappers to not touch 
the affected code.

```rust
let traverse_limit_inner = ctx
    .config::<millimeter>("traverse.limit.inner")
    .default(0.0)
    .on_changed(Self::on_traverse_limit_inner_changed)
    .register()?;
```

##### Registerig A State Property

Similarl to config we can register a schema like: 

```yaml
state:
  is_homed: !boolean
  is_going_in: !boolean
  is_going_out: !boolean
```

```rust
let traverse_is_homed     = ctx.state::<bool>("traverse.is_homed").register()?,
let traverse_is_going_in  = ctx.state::<bool>("traverse.is_going_in").register()?,
let traverse_is_going_out = ctx.state::<bool>("traverse.is_going_out").register()?,
```

##### Registerig A Measurement

Pretty much the same here:

```yaml
measurements:
  traverse:
    position: !length
      nullable: true

  puller:
    speed: !meter_per_minute
```

```rust
let traverse_position = ctx
    .measurement::<Option<millimeter>>("traverse.position")
    .register()?,

let puller_speed = ctx
    .measurement::<meter_per_minute>("puller.speed")
    .register()?,
```

##### Registerig A Command

```yaml
commands:
  enter_standby_mode: !command
  enter_hold_mode: !command

  traverse:
    goto_home: !command
    goto_limit_inner: !command
```

```rust
ctx.command("set_mode.standby")
    .execute(WinderV1::cmd_mode_standby)
    .register()?;

ctx.command("set_mode.hold")
    .can_execute(WinderV1::can_hold)
    .execute(WinderV1::cmd_mode_hold)
    .register()?;

ctx.command("traverse.goto_home")
    .can_execute(WinderV1::can_go_home)
    .execute(WinderV1::cmd_traverse_home)
    .register()?;

ctx.command("traverse.goto_limit_inner")
    .can_execute(WinderV1::can_go_in)
    .execute(WinderV1::cmd_goto_limit_inner)
    .register()?;
```

A registered command doesn't return anything since it's simply a definition.
It requires the function we defined earlier as input in execute, additionally
can_execute can be provided which allows to restrict the commands invocation 
based on machine state. The signature is: fn(&MyMachine) -> bool.

## 6. Next steps

You have successfully completed the initial migration of a machine!
Now this is as mentioned simply a bare bones port to avoid changing as much as possible.
However you will notice there is a lot of duplication and redundancy. For example
there is a high probability that you have the same ConfigProperty as StateProperty since
originally StateEvent was used to broadcast the configs current value/state and the mutation
was used to change it. Now config property achieves both and is actually intended to be
used directly as property inside the code. 

### 6.1 Injecting Config Properties and Removing duplicate state properties

```rust
#[derive(Debug)]
pub struct TraverseController {
    limit_inner: Length,
    limit_outer: Length,
    step_size: Length,
    padding: Length,
    // ...
}
```

Currently we internally use a config mutations on_changed callback to change these values
and a state property to expose the current value. In order for the config property to 
handle both responsibilities we need to hijack the old regular values at the source
and replace them with config properties as seen below.

```rust
#[derive(Debug)]
pub struct TraverseController {
    limit_outer: ConfigProperty<Length>,
    limit_inner: ConfigProperty<Length>,
    step_size: ConfigProperty<Length>,
    padding: ConfigProperty<Length>,
    // ...
}
```

#### 6.1.1 Adjusting initializers

However this has several side effects. In many cases the `new()` function has to be 
adjusted or `#[derive(Default)]` be replaced with a nen initializer that accepts the 
properties which will then be passed from the `build()` function.

Before:

```rust
pub fn new(
    limit_inner: Length, 
    limit_outer: Length, 
    // ...
) -> Self {
    Self {
        limit_inner,
        limit_outer,
        step_size: Length::new::<millimeter>(1.75),
        padding: Length::new::<millimeter>(0.88),
```

After:

```rust
pub fn new(
    limit_inner: ConfigProperty<Length>,
    limit_outer: ConfigProperty<Length>,
    step_size: ConfigProperty<Length>,
    padding: ConfigProperty<Length>,
    // ...
    ) -> Self {
    Self {
        limit_inner,
        limit_outer,
        step_size,
        padding,
        // ...
    }
```

#### 6.1.2 Refactoring usage or properties

Since properties are wrapper around values they expose getters and setters.
Thus in old places signatures like below will not work anymore.

```rust
if self.position <= self.limit_inner + self.padding {
    self.state = State::Traversing(TraversingState::TraversingOut);
}

self.limit_inner = limit;
```

We need to use the appropiate getters and setters:

```rust
if self.position <= self.limit_inner.get() + self.padding.get() {
    self.state = State::Traversing(TraversingState::TraversingOut);
}

self.limit_inner.set(limit)?;
```

Notice how config properties can fail when setting and thus must be accounted for!

All property types expose:

```rust
.get_ref() -> &T
.set(T) -> Result<(), ConfigPropertyWriteError>
```

If the type is Copy it addionally exposes:

```rust
.get() -> T
```

uom (units of measurements) expose additional getters and setters:

```rust
.get_as::<U>() -> f64
.set_as::<U>() -> Result<(), ConfigPropertyWriteError>
```

#### 6.1.3 Removing duplicate state properties

Now that the config property lives at the source it automatically contains the up to date
value, therefore the state property with the matching name can be removed now.

#### 6.1.4 Removing redundant callbacks

In most cases our on_changed callbacks simply wrote the value to the source which is now
our property, therefore in most cases they can be removed. In other cases where side effects
are desired the callback can still be used.

### 6.2 Injecting State Properties 

Just like config properties, state properties are intended to be used as direct replacements
for the values they represent.

```rust
pub struct LaserMachine {
    in_tolerance: bool,
    // ...
}
```

becomes:

```rust
pub struct LaserMachine {
    in_tolerance: StateProperty<bool>,
    // ...
}
```

All other migrations as mentioned in Config Properties apply here too, same setters
same getters.

### 6.3 Injecting Measurements

Same process really

```rust
pub struct LaserMachine {
    diameter: Length,
    // ...
}
```

becomes:

```rust
pub struct LaserMachine {
    diameter: Measurement<Length>,
    // ...
}
```

### 6.4 Merging commands

For easier migration we simply wrapped the initial value and used a cmd_* prefix to solve
the collision, now we can merge both into one command function. Example:

Instead of:

```rust
pub fn cmd_traverse_goto_home(&mut self) -> CommandExecuteResult {
    self.traverse_goto_home();
    Ok(())
}

pub fn traverse_goto_home(&mut self) {
    if self.can_go_home() {
        self.traverse_controller.goto_home();
    }
}

ctx.command("traverse.goto_home")
    .execute(WinderV1::cmd_traverse_goto_home)
    .register()?;
```

Do:

```rust
pub fn traverse_goto_home(&mut self) -> CommandExecuteResult {
    self.traverse_controller.goto_home();
    Ok(())
}

ctx.command("traverse.goto_home")
    .can_exeucte(WinderV1::can_go_home)
    .execute(WinderV1::cmd_traverse_goto_home)
    .register()?;
```

### 6.5 Removing the migration helpers

since all properties are now in their correct place 

`update_measurements` and `update_state` helper functions are no 
longer useful and can be removed.

## 7. The End

You have successfully finished the migration, congratulations!