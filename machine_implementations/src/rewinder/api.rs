use crate::{MachineApi, MachineMessage, MachineValues};
use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

use super::{Mode, TraverseStart};

#[derive(Deserialize, Serialize)]
pub enum Mutation {
    SetMode(Mode),
    SetPullerTargetSpeed(f64),
    SetTakeupSpoolDiameter(f64),
    SetSourceSpoolDiameter(f64),
    SetTakeupTensionArmControl(TensionArmControlState),
    SetSourceTensionArmControl(TensionArmControlState),
    SetPrepareControl(PrepareControlState),
    HardStop,
    SetRewindAutomaticRequiredMeters(f64),
    SetRewindAutomaticAction(RewindAutomaticActionMode),
    ResetRewindProgress,
    ZeroTakeupTensionArm,
    ZeroSourceTensionArm,
    SetTraverseLimitOuter(f64),
    SetTraverseLimitInner(f64),
    SetTraverseStart(TraverseStart),
    SetTraverseStartPosition(f64),
    SetTraverseStepSize(f64),
    SetTraversePadding(f64),
    GotoTraverseLimitOuter,
    GotoTraverseLimitInner,
    GotoTraverseStartPosition,
    GotoTraverseHome,
    EnableTraverseLaserpointer(bool),
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct LiveValuesEvent {
    pub traverse_position: Option<f64>,
    pub puller_speed: f64,
    pub takeup_spool_rpm: f64,
    pub source_spool_rpm: f64,
    pub takeup_tension_arm_angle: f64,
    pub source_tension_arm_angle: f64,
    pub rewind_progress: f64,
}

impl LiveValuesEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("LiveValuesEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct HardStopEvent {
    pub reason: String,
    pub source_angle: Option<f64>,
    pub takeup_angle: Option<f64>,
    pub source_min_angle: f64,
    pub source_max_angle: f64,
    pub takeup_min_angle: f64,
    pub takeup_max_angle: f64,
    pub source_out_of_range: bool,
    pub takeup_out_of_range: bool,
}

impl HardStopEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("HardStopEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub is_default_state: bool,
    pub mode_state: ModeState,
    pub traverse_state: TraverseState,
    pub puller_state: PullerState,
    pub takeup_spool_state: TakeupSpoolState,
    pub source_spool_state: SourceSpoolState,
    pub rewind_automatic_action_state: RewindAutomaticActionState,
    pub takeup_tension_arm_state: TensionArmState,
    pub source_tension_arm_state: TensionArmState,
    pub takeup_tension_arm_control_state: TensionArmControlState,
    pub source_tension_arm_control_state: TensionArmControlState,
    pub prepare_control_state: PrepareControlState,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct ModeState {
    pub mode: Mode,
    pub can_rewind: bool,
    pub motion_stopped: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TraverseState {
    pub limit_inner: f64,
    pub limit_outer: f64,
    pub position_in: f64,
    pub position_out: f64,
    pub start: TraverseStart,
    pub start_position: f64,
    pub custom_start_position: f64,
    pub is_going_in: bool,
    pub is_going_out: bool,
    pub is_going_to_start: bool,
    pub is_homed: bool,
    pub is_going_home: bool,
    pub is_traversing: bool,
    pub step_size: f64,
    pub padding: f64,
    pub laserpointer: bool,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct PullerState {
    pub target_speed: f64,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TakeupSpoolState {
    pub diameter_mm: Option<f64>,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct SourceSpoolState {
    pub diameter_mm: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
pub struct TensionArmControlState {
    pub hard_min_angle: f64,
    pub hard_max_angle: f64,
    pub start_min_angle: f64,
    pub start_max_angle: f64,
    pub target_angle: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Default)]
pub struct PrepareControlState {
    pub tolerance_angle: f64,
    pub settle_rate: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub enum RewindAutomaticActionMode {
    #[default]
    NoAction,
    Hold,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct RewindAutomaticActionState {
    pub required_meters: f64,
    pub mode: RewindAutomaticActionMode,
}

#[derive(Serialize, Debug, Clone, Default)]
pub struct TensionArmState {
    pub zeroed: bool,
}

pub enum RewinderEvents {
    HardStop(Event<HardStopEvent>),
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

#[derive(Debug)]
pub struct RewinderNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<RewinderEvents> for RewinderNamespace {
    #[instrument(skip_all)]
    fn emit(&mut self, events: RewinderEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

impl CacheableEvents<Self> for RewinderEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            Self::HardStop(event) => event.into(),
            Self::LiveValues(event) => event.into(),
            Self::State(event) => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        let cache_first_and_last = cache_first_and_last_event();
        match self {
            Self::HardStop(_) => Box::new(|_, _| {}),
            Self::LiveValues(_) => cache_first_and_last,
            Self::State(_) => cache_first_and_last,
        }
    }
}

impl MachineApi for super::Rewinder {
    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.api_sender.clone()
    }

    fn api_mutate(&mut self, request_body: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(request_body)?;
        match mutation {
            Mutation::SetMode(mode) => self.set_mode(&mode),
            Mutation::SetPullerTargetSpeed(speed) => self.puller_set_target_speed(speed),
            Mutation::SetTakeupSpoolDiameter(value) => self.takeup_spool_set_diameter(value),
            Mutation::SetSourceSpoolDiameter(value) => self.source_spool_set_diameter(value),
            Mutation::SetTakeupTensionArmControl(config) => {
                self.set_takeup_tension_arm_control(config)
            }
            Mutation::SetSourceTensionArmControl(config) => {
                self.set_source_tension_arm_control(config)
            }
            Mutation::SetPrepareControl(config) => self.set_prepare_control(config),
            Mutation::HardStop => self.manual_hard_stop(),
            Mutation::SetRewindAutomaticRequiredMeters(meters) => {
                self.set_rewind_automatic_required_meters(meters)
            }
            Mutation::SetRewindAutomaticAction(mode) => self.set_rewind_automatic_action(mode),
            Mutation::ResetRewindProgress => self.reset_rewind_progress(std::time::Instant::now()),
            Mutation::ZeroTakeupTensionArm => self.takeup_tension_arm_zero(),
            Mutation::ZeroSourceTensionArm => self.source_tension_arm_zero(),
            Mutation::SetTraverseLimitOuter(limit) => self.traverse_set_limit_outer(limit),
            Mutation::SetTraverseLimitInner(limit) => self.traverse_set_limit_inner(limit),
            Mutation::SetTraverseStart(start) => self.traverse_set_start(start),
            Mutation::SetTraverseStartPosition(position) => {
                self.traverse_set_start_position(position)
            }
            Mutation::SetTraverseStepSize(size) => self.traverse_set_step_size(size),
            Mutation::SetTraversePadding(padding) => self.traverse_set_padding(padding),
            Mutation::GotoTraverseLimitOuter => self.traverse_goto_limit_outer(),
            Mutation::GotoTraverseLimitInner => self.traverse_goto_limit_inner(),
            Mutation::GotoTraverseStartPosition => self.traverse_goto_start_position(),
            Mutation::GotoTraverseHome => self.traverse_goto_home(),
            Mutation::EnableTraverseLaserpointer(enable) => self.set_laser(enable),
        }
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }

    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => crate::respond_values(
                sender,
                MachineValues {
                    state: serde_json::to_value(self.build_state_event())
                        .expect("Failed to serialize state"),
                    live_values: serde_json::to_value(self.get_live_values())
                        .expect("Failed to serialize live values"),
                },
            ),
        }
    }
}
