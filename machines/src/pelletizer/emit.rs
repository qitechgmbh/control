
use crate::pelletizer::Pelletizer;

use crate::pelletizer::api::{InverterLiveValues, InverterState, LiveValuesEvent, PelletMachineEvents, StateEvent};

use control_core::socketio::event::BuildEvent;

use control_core::socketio::namespace::NamespaceCacheingLogic;

impl Pelletizer
{
    pub fn emit_state(&mut self) 
    {
        let event = self.create_state_event().build();
        self.namespace.emit(PelletMachineEvents::State(event));
        self.emitted_default_state = true;
    }

    pub fn emit_live_values(&mut self)
    {
        let event = self.create_live_values_event().build();
        self.namespace.emit(PelletMachineEvents::LiveValues(event));
    }

    pub fn create_state_event(&self) -> StateEvent 
    {
        let inverter = smol::block_on(async {
            self.inverter.read().await
        });

        StateEvent 
        {
            is_default_state: !self.emitted_default_state,
            inverter_state: InverterState {
                running:            inverter.config.running,
                direction:          inverter.config.direction,
                frequency_target:   (inverter.config.frequency_target as f64) / 10.0,
                acceleration_level: inverter.config.acceleration_level,
                deceleration_level: inverter.config.deceleration_level,
            },
        }
    }

    pub fn create_live_values_event(&self) -> LiveValuesEvent
    {
        let inverter = smol::block_on(async {
            self.inverter.read().await
        });

        if let Some(status) = inverter.status
        {
            return LiveValuesEvent {
                inverter_values: InverterLiveValues {
                    voltage:     status.voltage.value,
                    current:     status.current.value,
                    temperature: status.temperature.value,
                    frequency:   status.frequency.value, 
                }
            };
        }
        
        LiveValuesEvent {
            inverter_values: InverterLiveValues {
                voltage:     0.0,
                current:     0.0,
                temperature: 0.0,
                frequency:   0.0, 
            }
        }
    }
}