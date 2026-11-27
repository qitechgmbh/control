//! Demo machines for testing without hardware (`DEMO_MACHINES=true`).
//!
//! Adds one production line (extruder, water bath, laser, winder, see [`line`]) that
//! looks like real machines to everything downstream of the runtime: [`DemoInjector`]
//! wraps the hub listeners and appends the demo machines' registration, property
//! records and 32 Hz measurements to every runtime report. The Socket.IO API and the
//! ClickHouse writer therefore handle demo data exactly like real data. On start,
//! [`backfill`] fills the last 30 days of history so the graphs can be tested with a
//! full database.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use qitech_framework::ConfigPropertyEvent;
use qitech_framework::ConfigPropertyEventRecord;
use qitech_framework::ConfigPropertyWriteOutcome;
use qitech_framework::Constraints;
use qitech_framework::MachineIdentification;
use qitech_framework::MachineInstanceIdentification;
use qitech_framework::MachineSchema;
use qitech_framework::RuntimeEvent;
use qitech_framework::RuntimeInitEvent;
use qitech_framework::RuntimeReport;
use qitech_framework::RuntimeRequestKind;
use qitech_framework::ScalarValue;
use qitech_framework::StatePropertyEvent;
use qitech_framework::StatePropertyEventRecord;
use qitech_framework_core::report::MeasurementSnapshot;
use qitech_framework_core::report::OperationCapability;
use qitech_framework_core::report::OperationOrigin;
use qitech_framework_core::schema::ScalarPropertyDefinition;
use qitech_framework_core::schema::ScalarPropertyKind;
use qitech_framework_hub::Listener;

pub mod backfill;
mod line;
mod signal;

use line::DemoMachine;
use line::Initial;
use signal::Base;
use signal::Noise;

/// Time constant (s) with which a measurement follows a changed target.
const FOLLOW_TAU_S: f64 = 45.0;

/// Whether `DEMO_MACHINES=true` is set.
pub fn enabled_by_env() -> bool {
    std::env::var("DEMO_MACHINES").is_ok_and(|v| v.eq_ignore_ascii_case("true") || v == "1")
}

/// Identities (`machine_*` tables) of the demo machines, also when demo mode is off.
pub fn identities() -> Vec<u64> {
    line::machines()
        .iter()
        .map(|m| m.ident().to_u64())
        .collect()
}

/// Shared with the API: which machines are demo machines, their pending mutations and
/// the backfill progress.
#[derive(Clone)]
pub struct DemoHandle {
    shared: Arc<Shared>,
}

struct Shared {
    idents: Vec<MachineInstanceIdentification>,
    requests: Mutex<Vec<RuntimeRequestKind>>,
    backfill: backfill::Progress,
}

impl DemoHandle {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Shared {
                idents: line::machines().iter().map(DemoMachine::ident).collect(),
                requests: Default::default(),
                backfill: Default::default(),
            }),
        }
    }

    pub fn is_demo(&self, ident: MachineInstanceIdentification) -> bool {
        self.shared.idents.contains(&ident)
    }

    /// Queues mutations of a demo machine; applied with the next runtime report.
    pub fn submit(&self, requests: Vec<RuntimeRequestKind>) {
        self.shared
            .requests
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .extend(requests);
    }

    fn take_requests(&self) -> Vec<RuntimeRequestKind> {
        std::mem::take(
            &mut *self
                .shared
                .requests
                .lock()
                .unwrap_or_else(|p| p.into_inner()),
        )
    }

    pub fn backfill(&self) -> &backfill::Progress {
        &self.shared.backfill
    }
}

/// Wraps the hub listeners and adds the demo machines to every report they receive.
pub struct DemoInjector {
    inner: Vec<Box<dyn Listener>>,
    handle: DemoHandle,
    schemas: HashMap<MachineIdentification, MachineSchema>,
    machines: Vec<LiveMachine>,
    noise: Noise,
    last_ms: Option<i64>,
}

impl DemoInjector {
    pub fn new(handle: DemoHandle) -> Self {
        Self {
            inner: Vec::new(),
            handle,
            schemas: HashMap::new(),
            machines: Vec::new(),
            noise: Noise::new(0x5eed_cafe),
            last_ms: None,
        }
    }

    pub fn listener<L: Listener + 'static>(mut self, listener: L) -> Self {
        self.inner.push(Box::new(listener));
        self
    }

    /// Builds the demo machines once the runtime has synced all their schemas.
    /// Returns whether they were built by this call.
    fn try_build(&mut self) -> bool {
        if !self.machines.is_empty() {
            return false;
        }
        let defs = line::machines();
        if !defs.iter().all(|m| self.schemas.contains_key(&m.machine)) {
            return false;
        }
        self.machines = defs
            .iter()
            .map(|def| LiveMachine::new(def, self.schemas[&def.machine].clone()))
            .collect();
        tracing::info!(machines = self.machines.len(), "demo machines added");
        true
    }

    fn inject(&mut self, report: &mut RuntimeReport) {
        let built = self.try_build();
        if self.machines.is_empty() {
            return;
        }

        let now_ms = report.timestamp.timestamp_millis();
        let dt = self
            .last_ms
            .map_or(0.0, |last| (now_ms - last).max(0) as f64 / 1000.0);
        self.last_ms = Some(now_ms);

        if built {
            for machine in &self.machines {
                machine.announce(report);
            }
        }

        for request in self.handle.take_requests() {
            let target = match &request {
                RuntimeRequestKind::SetConfigProperty { target, .. }
                | RuntimeRequestKind::ExecuteCommand { target, .. } => *target,
                _ => continue,
            };
            if let Some(machine) = self.machines.iter_mut().find(|m| m.ident == target) {
                machine.apply(request, report);
            }
        }

        let t = signal::seconds(now_ms);
        for machine in &mut self.machines {
            machine.sample(t, dt, &mut self.noise, report);
        }
    }
}

impl Listener for DemoInjector {
    fn on_schema_sync(&mut self, schema: &MachineSchema) {
        if line::machines()
            .iter()
            .any(|m| m.machine == schema.identification)
        {
            self.schemas.insert(schema.identification, schema.clone());
        }
        for listener in &mut self.inner {
            listener.on_schema_sync(schema);
        }
    }

    fn on_init_event_received(&mut self, event: &RuntimeInitEvent) {
        for listener in &mut self.inner {
            listener.on_init_event_received(event);
        }
    }

    fn on_report_received(&mut self, report: &RuntimeReport) {
        let mut report = report.clone();
        self.inject(&mut report);
        for listener in &mut self.inner {
            listener.on_report_received(&report);
        }
    }

    fn on_runtime_disconnected(&mut self) {
        for listener in &mut self.inner {
            listener.on_runtime_disconnected();
        }
    }
}

/// A demo machine while the backend runs.
struct LiveMachine {
    def: &'static DemoMachine,
    ident: MachineInstanceIdentification,
    schema: MachineSchema,
    config: HashMap<String, ScalarValue>,
    state: HashMap<String, ScalarValue>,
    /// Current base of each measurement that follows a config property (smoothed).
    bases: Vec<Option<f64>>,
}

impl LiveMachine {
    fn new(def: &'static DemoMachine, schema: MachineSchema) -> Self {
        let config = schema
            .config_properties
            .iter()
            .map(|(path, d)| (path.clone(), initial_value(d, def.config_initial(path))))
            .collect();
        let state = schema
            .state_properties
            .iter()
            .map(|(path, d)| (path.clone(), initial_value(d, def.state_initial(path))))
            .collect();

        let mut machine = Self {
            def,
            ident: def.ident(),
            schema,
            config,
            state,
            bases: Vec::new(),
        };
        machine.bases = def
            .measurements
            .iter()
            .map(|(_, s)| machine.target_base(s))
            .collect();
        machine
    }

    /// Unsmoothed base of a signal that follows a config property.
    fn target_base(&self, signal: &signal::Signal) -> Option<f64> {
        let signal::Signal::Wave {
            base: Base::Config { path, offset },
            ..
        } = signal
        else {
            return None;
        };
        let value = match self.config.get(*path) {
            Some(ScalarValue::Float(v)) => *v,
            Some(ScalarValue::Integer(v)) => *v as f64,
            Some(ScalarValue::Boolean(b)) => *b as u8 as f64,
            _ => self.def.config_float(path),
        };
        Some(value + offset)
    }

    /// Registers the machine and all its properties, like a freshly built machine.
    fn announce(&self, report: &mut RuntimeReport) {
        let timestamp = report.timestamp;
        report
            .events
            .push(RuntimeEvent::AddedMachine { ident: self.ident });

        for path in self.schema.config_properties.keys() {
            report
                .machines
                .config_property_records
                .push(ConfigPropertyEventRecord {
                    timestamp,
                    machine: self.ident,
                    path: path.clone(),
                    event: ConfigPropertyEvent::Registered {
                        default: self.config[path].clone(),
                        capability: OperationCapability::Allowed,
                        constraints: Constraints::None,
                    },
                });
        }

        for path in self.schema.state_properties.keys() {
            report
                .machines
                .state_property_records
                .push(StatePropertyEventRecord {
                    timestamp,
                    machine: self.ident,
                    path: path.clone(),
                    event: StatePropertyEvent::Registered {
                        value: self.state[path].clone(),
                    },
                });
        }
    }

    /// Accepts every config write; commands `a.b` set the enum state `a` to `b`
    /// (e.g. `mode.extrude`), and `x.enable`/`x.disable` the boolean state `x.enabled`.
    fn apply(&mut self, request: RuntimeRequestKind, report: &mut RuntimeReport) {
        let timestamp = report.timestamp;
        match request {
            RuntimeRequestKind::SetConfigProperty { path, value, .. } => {
                let Some(current) = self.config.get_mut(&path) else {
                    return;
                };
                let changed = *current != value;
                *current = value.clone();
                report
                    .machines
                    .config_property_records
                    .push(ConfigPropertyEventRecord {
                        timestamp,
                        machine: self.ident,
                        path,
                        event: ConfigPropertyEvent::Written {
                            value,
                            origin: OperationOrigin::Request { request_id: 0 },
                            outcome: ConfigPropertyWriteOutcome::Accepted { changed },
                        },
                    });
            }
            RuntimeRequestKind::ExecuteCommand { path, .. } => {
                let Some((prefix, action)) = path.rsplit_once('.') else {
                    return;
                };
                let change = match action {
                    "enable" | "disable" => Some((
                        format!("{prefix}.enabled"),
                        ScalarValue::Boolean(action == "enable"),
                    )),
                    _ => match self.schema.state_properties.get(prefix).map(|d| &d.kind) {
                        Some(ScalarPropertyKind::Enum { variants })
                            if variants.contains_name(action) =>
                        {
                            Some((prefix.to_owned(), ScalarValue::Enum(variant_ident(action))))
                        }
                        _ => None,
                    },
                };
                let Some((state_path, value)) = change else {
                    return;
                };
                let Some(current) = self.state.get_mut(&state_path) else {
                    return;
                };
                *current = value.clone();
                report
                    .machines
                    .state_property_records
                    .push(StatePropertyEventRecord {
                        timestamp,
                        machine: self.ident,
                        path: state_path,
                        event: StatePropertyEvent::ValueChanged { value },
                    });
            }
            _ => {}
        }
    }

    fn sample(&mut self, t: f64, dt: f64, noise: &mut Noise, report: &mut RuntimeReport) {
        let alpha = 1.0 - (-dt / FOLLOW_TAU_S).exp();
        for (i, (path, signal)) in self.def.measurements.iter().enumerate() {
            let base = match (self.bases[i], self.target_base(signal)) {
                (Some(current), Some(target)) => {
                    let next = current + (target - current) * alpha;
                    self.bases[i] = Some(next);
                    next
                }
                _ => match signal {
                    signal::Signal::Wave {
                        base: Base::Const(b),
                        ..
                    } => *b,
                    _ => 0.0,
                },
            };
            let value = signal.eval(t, base, signal::phase(i), noise.next());
            report
                .machines
                .measurement_snapshots
                .push(MeasurementSnapshot {
                    machine: self.ident,
                    path: (*path).to_owned(),
                    value: Some(value),
                });
        }
    }
}

/// Value a property starts with: the line's setting, else a neutral default of its type.
fn initial_value(def: &ScalarPropertyDefinition, initial: Option<Initial>) -> ScalarValue {
    match (initial, &def.kind) {
        (Some(Initial::Float(v)), ScalarPropertyKind::Integer) => ScalarValue::Integer(v as i64),
        (Some(Initial::Float(v)), _) => ScalarValue::Float(v),
        (Some(Initial::Bool(b)), _) => ScalarValue::Boolean(b),
        (Some(Initial::Enum(v)), _) => ScalarValue::Enum(variant_ident(v)),
        (None, _) if def.nullable => ScalarValue::Null,
        (None, ScalarPropertyKind::Enum { variants }) => {
            ScalarValue::Enum(variants.first().0.clone())
        }
        (None, ScalarPropertyKind::String) => ScalarValue::String(String::new()),
        (None, ScalarPropertyKind::Boolean) => ScalarValue::Boolean(false),
        (None, ScalarPropertyKind::Integer) => ScalarValue::Integer(0),
        (None, ScalarPropertyKind::Float { .. }) => ScalarValue::Float(0.0),
    }
}

/// The Rust variant ident of an enum value named in the schema (`not_started` →
/// `NotStarted`). Real machines register enum properties and report enum states in this
/// spelling (only API writes echo the snake_case name), and the legacy adapters and the
/// frontend depend on it, e.g. the extruder's `mode` is passed to the UI verbatim.
pub fn variant_ident(name: &str) -> String {
    name.split('_')
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema(def: &DemoMachine) -> MachineSchema {
        MachineSchema::parse_str(def.schema).expect("schema should parse")
    }

    /// Every schema measurement has a signal, and every setting names a real property,
    /// so a schema change cannot silently leave the demo line half-filled.
    #[test]
    fn line_matches_schemas() {
        for def in line::machines() {
            let schema = schema(def);
            let mut signals: Vec<_> = def.measurements.iter().map(|(p, _)| *p).collect();
            let mut measurements: Vec<_> = schema.measurements.keys().map(String::as_str).collect();
            signals.sort_unstable();
            measurements.sort_unstable();
            assert_eq!(signals, measurements, "{}", schema.name);

            for (path, _) in def.config {
                assert!(
                    schema.config_properties.contains_key(*path),
                    "{} config {path}",
                    schema.name
                );
            }
            for (path, _) in def.state {
                assert!(
                    schema.state_properties.contains_key(*path),
                    "{} state {path}",
                    schema.name
                );
            }
            for (_, signal) in def.measurements {
                if let signal::Signal::Wave {
                    base: Base::Config { path, .. },
                    ..
                } = signal
                {
                    assert!(
                        schema.config_properties.contains_key(*path),
                        "{} follows {path}",
                        schema.name
                    );
                }
            }
        }
    }

    #[test]
    fn legacy_adapters_emit_for_demo_machines() {
        use crate::api::legacy_adapter_events;

        for def in line::machines() {
            let machine = LiveMachine::new(def, schema(def));
            let mut report = RuntimeReport::default();
            machine.announce(&mut report);
            let mut machine = machine;
            machine.sample(1000.0, 0.03, &mut Noise::new(1), &mut report);

            let (state, live) = legacy_adapter_events(def.ident(), &schema(def), &report);
            assert!(state.is_some(), "no state event for {}", def.machine);
            assert!(live.is_some(), "no live values event for {}", def.machine);
        }
    }

    #[test]
    fn variant_idents() {
        assert_eq!(variant_ident("not_started"), "NotStarted");
        assert_eq!(variant_ident("observer_pi"), "ObserverPi");
        assert_eq!(variant_ident("extrude"), "Extrude");
    }

    /// The frontend validates the extruder's state event; it expects PascalCase variants.
    #[test]
    fn extruder_state_event_uses_variant_idents() {
        use crate::api::legacy_adapter_events;

        let def = &line::machines()[0];
        let machine = LiveMachine::new(def, schema(def));
        let mut report = RuntimeReport::default();
        machine.announce(&mut report);
        let (state, _) = legacy_adapter_events(def.ident(), &schema(def), &report);
        let state = state.expect("state event");
        assert_eq!(state["mode_state"]["mode"], "Extrude");
        assert_eq!(state["extruder_settings_state"]["heating_algorithm"], "Pid");
        assert_eq!(state["regulation_state"]["uses_rpm"], true);
        assert_eq!(state["rotation_state"]["forward"], true);
    }

    #[test]
    fn commands_switch_enum_state() {
        let def = &line::machines()[0];
        let mut machine = LiveMachine::new(def, schema(def));
        let mut report = RuntimeReport::default();
        machine.apply(
            RuntimeRequestKind::ExecuteCommand {
                target: def.ident(),
                path: "mode.standby".into(),
            },
            &mut report,
        );
        assert_eq!(machine.state["mode"], ScalarValue::Enum("Standby".into()));
        assert_eq!(report.machines.state_property_records.len(), 1);
    }
}
