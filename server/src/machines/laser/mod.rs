use crate::{
    machines::{
        laser::api::ConnectedMachineState, winder2::{self, Winder2}, MACHINE_LASER_V1, VENDOR_QITECH
    },
    serial::devices::laser::Laser,
};
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::{
    helpers::hasher_serializer::check_hash_different,
    machines::{
        downcast_machine, identification::{MachineIdentification, MachineIdentificationUnique}, manager::MachineManager, ConnectedMachine, ConnectedMachineData, Machine
    },
    socketio::namespace::NamespaceCacheingLogic,
};
use futures::executor::block_on;
use smol::lock::Mutex;
use smol::lock::RwLock;
use std::{
    any::Any,
    sync::{Arc, Weak},
    time::Instant,
};
use uom::si::{f64::Length, length::millimeter};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Machine)]
pub struct LaserMachine {
    // drivers
    laser: Arc<RwLock<Laser>>,

    // socketio
    namespace: LaserMachineNamespace,
    last_measurement_emit: Instant,

    // machine connection
    pub machine_manager: Weak<RwLock<MachineManager>>,
    pub machine_identification_unique: MachineIdentificationUnique,

    // connected machines
    pub connected_winder: Option<ConnectedMachine<Weak<Mutex<Winder2>>>>,

    //laser target configuration
    laser_target: LaserTarget,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
    last_state_event: Option<StateEvent>,
}

impl LaserMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_LASER_V1,
    };
}

impl LaserMachine {
    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let diameter = smol::block_on(async {
            self.laser
                .read()
                .await
                .get_data()
                .await
                .map(|laser_data| laser_data.diameter.get::<millimeter>())
        });
        let live_values = LiveValuesEvent {
            diameter: diameter.unwrap_or(0.0),
        };
        self.namespace
            .emit(LaserEvents::LiveValues(live_values.build()));
    }

    pub fn build_state_event(&self) -> StateEvent {
        let laser = LaserState {
            higher_tolerance: self.laser_target.higher_tolerance.get::<millimeter>(),
            lower_tolerance: self.laser_target.lower_tolerance.get::<millimeter>(),
            target_diameter: self.laser_target.diameter.get::<millimeter>(),
        };

        StateEvent {
            is_default_state: false,
            laser_state: laser,
        }
    }

    pub fn maybe_emit_state_event(&mut self) {
        let new_state: StateEvent = self.build_state_event();
        let old_state: &StateEvent = match &self.last_state_event {
            Some(old_state) => old_state,
            None => {
                self.emit_state();
                return;
            }
        };

        let should_emit = check_hash_different(&new_state, old_state);
        if should_emit {
            let event = &new_state.build();
            self.last_state_event = Some(new_state);
            self.namespace.emit(LaserEvents::State(event.clone()));
        }
    }

    pub fn emit_state(&mut self) {
        let state = StateEvent {
            is_default_state: !std::mem::replace(&mut self.emitted_default_state, true),
            laser_state: LaserState {
                higher_tolerance: self.laser_target.higher_tolerance.get::<millimeter>(),
                lower_tolerance: self.laser_target.lower_tolerance.get::<millimeter>(),
                target_diameter: self.laser_target.diameter.get::<millimeter>(),
            },
            connected_winder_state: ConnectedMachineState {
                machine_identification_unique: self.connected_winder.as_ref().map(
                    |connected_machine| {
                        ConnectedMachineData::from(connected_machine)
                            .machine_identification_unique
                            .clone()
                    },
                ),
                is_available: self
                    .connected_winder
                    .as_ref()
                    .map(|connected_machine| {
                        ConnectedMachineData::from(connected_machine).is_available
                    })
                    .unwrap_or(false),
            },
        };

        self.namespace.emit(LaserEvents::State(state.build()));
    }

    pub fn set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.laser_target.higher_tolerance = Length::new::<millimeter>(higher_tolerance);
    }

    pub fn set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.laser_target.lower_tolerance = Length::new::<millimeter>(lower_tolerance);
    }

    pub fn set_target_diameter(&mut self, target_diameter: f64) {
        self.laser_target.diameter = Length::new::<millimeter>(target_diameter);
        self.emit_state();
    }
}

/// implement machine connection
impl LaserMachine {
    /// set connected winder
    pub fn set_connected_winder(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            Winder2::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => return,
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let winder2_weak = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let winder2_weak = match winder2_weak {
            Some(winder2_weak) => winder2_weak,
            None => return,
        };
        let winder2_strong = match winder2_weak.upgrade() {
            Some(winder2_strong) => winder2_strong,
            None => return,
        };

        let winder2: Arc<Mutex<Winder2>> = block_on(downcast_machine::<Winder2>(winder2_strong))
            .expect("failed downcasting machine");

        let machine = Arc::downgrade(&winder2);

        self.connected_winder = Some(ConnectedMachine {
            machine_identification_unique,
            machine: machine.clone(),
        });

        self.emit_state();

        self.reverse_connect_winder();
    }

    /// disconnect winder
    pub fn disconnect_winder(
        &mut self,
        machine_identification_unique: MachineIdentificationUnique,
    ) {
        if !matches!(
            machine_identification_unique.machine_identification,
            Winder2::MACHINE_IDENTIFICATION
        ) {
            return;
        }
        if let Some(connected) = &self.connected_winder {
            if let Some(winder_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut winder = winder_arc.lock().await;
                    if winder.connected_laser.is_some() {
                        winder.connected_laser = None;
                        winder.emit_state();
                    }
                };
                smol::spawn(future).detach();
            }
        }
        self.connected_winder = None;
        self.emit_state();
    }

    /// initiate connection from laser to winder
    pub fn reverse_connect_winder(&mut self) {
        let machine_identification_unique = self.machine_identification_unique.clone();
        if let Some(connected) = &self.connected_winder {
            if let Some(winder_arc) = connected.machine.upgrade() {
                let future = async move {
                    let mut winder = winder_arc.lock().await;
                    if winder.connected_laser.is_none() {
                        winder.set_connected_laser(machine_identification_unique);
                    }
                };
                smol::spawn(future).detach();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct LaserTarget {
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length,
}
