use std::time::{Duration, Instant};

// external deps
use anyhow::{Error};
use serde_json::Value;
use smol::channel::Sender;

// local deps
use control_core::socketio::{
    event::{
        BuildEvent, 
        Event, 
        GenericEvent
    }, 
    namespace::{
        CacheFn, 
        CacheableEvents, 
        Namespace, 
        NamespaceCacheingLogic, 
        cache_first_and_last_event
    }
};

use crate::{
    AsyncThreadMessage, 
    Machine, 
    MachineAct, 
    MachineApi, 
    MachineMessage, 
    MachineNewParams, 
    MachineNewTrait, 
    MachineValues, 
    machine_identification::MachineIdentificationUnique, 
};

use crate::stahlwerk::{
    r#abstract::{
        BaseMachine, 
        NamespaceImpl
    }
};

use super::{ 
    DerivedMachine,
    api::{
        LiveValuesEvent,
        StateEvent,
        Mutation,
    }
};

impl DerivedMachine
{
    /// Emit live values data event with the current sine wave amplitude
    pub fn emit_live_values(&mut self) 
    {
        let event = self.get_live_values().build();
        self.base.namespace.emit(Events::LiveValues(event));
    }

    /// Emit the current state of the mock machine only if values have changed
    pub fn emit_state(&mut self) 
    {
        let state = self.get_state();
        let event = state.build();
        self.base.namespace.emit(Events::State(event));
        self.base.emitted_default_state = true;
    }
}

impl MachineNewTrait for DerivedMachine 
{
    fn new(params: &MachineNewParams<'_, '_, '_, '_, '_, '_, '_>) -> Result<Self, Error>
    where
        Self: Sized 
    {
        let (api_sender, api_receiver) = smol::channel::unbounded();

        let base = BaseMachine {
            machine_uid: params.get_machine_identification_unique(),
            main_sender: params.main_thread_channel.clone(),
            namespace:   NamespaceImpl::new(params.namespace.clone()),
            last_measurement_emit: Instant::now(),
            api_sender,
            api_receiver,
            emitted_default_state: false,
        };

        let mut instance = Self::new(base);

        instance.emit_state();

        Ok(instance)
    }
}

impl Machine for DerivedMachine
{
    fn get_machine_identification_unique(&self) -> MachineIdentificationUnique 
    {
        self.base.machine_uid.clone()
    }

    fn get_main_sender(&self) -> Option<Sender<AsyncThreadMessage>> 
    {
        self.base.main_sender.clone()
    }
}

impl MachineApi for DerivedMachine
{
    fn api_get_sender(&self) -> Sender<MachineMessage> 
    {
        self.base.api_sender.clone()
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> 
    {
        self.base.namespace.namespace.clone()
    }
    
    fn api_mutate(&mut self, value: Value) -> Result<(), Error> 
    {
        let mutation: Mutation = serde_json::from_value(value)?;
        self.mutate(mutation);
        Ok(())
    }
}

impl MachineAct for DerivedMachine
{
    fn act_machine_message(&mut self, msg: MachineMessage) 
    {
        use MachineMessage::*;
        use crate::MachineApi;

        match msg 
        {
            SubscribeNamespace(namespace) => 
            {
                self.base.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            UnsubscribeNamespace => self.base.namespace.namespace = None,
            HttpApiJsonRequest(value) => _ = self.api_mutate(value),
            ConnectToMachine(_)   => {}
            DisconnectMachine(_)  => {}
            RequestValues(sender) => 
            {
                sender
                    .send_blocking(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::to_value(self.get_live_values())
                            .expect("Failed to serialize live values"),
                    })
                    .expect("Failed to send values");

                sender.close();
            }
        }
    }

    fn act(&mut self, now: Instant) 
    {
        let target_duration: Duration = Duration::from_secs_f64(1.0 / 30.0);

        match self.base.api_receiver.try_recv() 
        {
            Ok(msg) => _ = self.act_machine_message(msg),
            Err(_)  => (),
        };

        self.act_ext(now);

        let now = Instant::now();

        // more than 33ms have passed since last emit (30 "fps" target)
        if now.duration_since(self.base.last_measurement_emit) > target_duration 
        {
            self.emit_state();
            self.emit_live_values();
            self.base.last_measurement_emit = now;
        }
    }
}

impl LiveValuesEvent 
{
    pub fn build(&self) -> Event<Self> 
    {
        Event::new("LiveValuesEvent", self.clone())
    }
}

pub enum Events 
{
    LiveValues(Event<LiveValuesEvent>),
    State(Event<StateEvent>),
}

impl CacheableEvents<Self> for Events 
{
    fn event_value(&self) -> GenericEvent 
    {
        match self 
        {
            Self::LiveValues(event) => event.into(),
            Self::State(event)      => event.into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn 
    {
        cache_first_and_last_event()
    }
}