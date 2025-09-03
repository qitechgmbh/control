use crate::{
    machines::{
        MACHINE_LASER_V1, VENDOR_QITECH,
        laser::api::{ConnectedMachineState, PidSettings, PidSettingsStates},
        winder2::Winder2,
    },
    serial::devices::laser::Laser,
};
use api::{LaserEvents, LaserMachineNamespace, LaserState, LiveValuesEvent, StateEvent};
use control_core::{
    machines::identification::{MachineIdentification, MachineIdentificationUnique},
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
use tracing::info;
use uom::si::{f64::Length, length::millimeter};

pub mod act;
pub mod api;
pub mod new;

#[derive(Debug, Machine)]
pub struct LaserMachine {
    machine_identification_unique: MachineIdentificationUnique,

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
    pub pid_settings: PidSettings,

    // laser values
    diameter: Length,
    x_diameter: Option<Length>,
    y_diameter: Option<Length>,
    roundness: Option<f64>,

    //laser target configuration
    laser_target: LaserTarget,

    /// Will be initialized as false and set to true by emit_state
    /// This way we can signal to the client that the first state emission is a default state
    emitted_default_state: bool,
}

impl LaserMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: MACHINE_LASER_V1,
    };

    ///diameter in mm
    pub fn emit_live_values(&mut self) {
        let diameter = self.diameter.get::<millimeter>();
        let x_diameter = self.x_diameter.map(|x| x.get::<millimeter>());
        let y_diameter = self.y_diameter.map(|y| y.get::<millimeter>());
        let roundness = self.roundness;

        let live_values = LiveValuesEvent {
            diameter,
            x_diameter,
            y_diameter,
            roundness,
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
            pid_settings: PidSettingsStates {
                speed: self.pid_settings.clone(),
            },
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
            pid_settings: PidSettingsStates {
                speed: self.pid_settings.clone(),
            },
        };

        self.namespace.emit(LaserEvents::State(state.build()));
    }

    pub fn set_higher_tolerance(&mut self, higher_tolerance: f64) {
        self.laser_target.higher_tolerance = Length::new::<millimeter>(higher_tolerance);
        self.emit_state();
    }

    pub fn set_lower_tolerance(&mut self, lower_tolerance: f64) {
        self.laser_target.lower_tolerance = Length::new::<millimeter>(lower_tolerance);
        self.emit_state();
    }

    pub fn set_target_diameter(&mut self, target_diameter: f64) {
        self.laser_target.diameter = Length::new::<millimeter>(target_diameter);
        self.emit_state();
    }

    ///
    /// Roundness = min(x, y) / max(x, y)
    ///
    fn calculate_roundness(&mut self) -> Option<f64> {
        match (self.x_diameter, self.y_diameter) {
            (Some(x), Some(y)) => {
                let x_val = x.get::<millimeter>();
                let y_val = y.get::<millimeter>();

                if x_val > 0.0 && y_val > 0.0 {
                    let roundness = f64::min(x_val, y_val) / f64::max(x_val, y_val);
                    Some(roundness)
                } else if x_val == 0.0 && y_val == 0.0 {
                    Some(0.0)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn update(&mut self) {
        let laser_data = smol::block_on(async { self.laser.read().await.get_data().await });
        self.diameter = Length::new::<millimeter>(
            laser_data
                .as_ref()
                .map(|data| data.diameter.get::<millimeter>())
                .unwrap_or(0.0),
        );

        self.x_diameter = laser_data
            .as_ref()
            .and_then(|data| data.x_axis.as_ref())
            .cloned();

        self.y_diameter = laser_data
            .as_ref()
            .and_then(|data| data.y_axis.as_ref())
            .cloned();

        self.roundness = self.calculate_roundness();
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
            tracing::trace!("Setting Connected Winder | did not match ID");
            return;
        }
        let machine_manager_arc = match self.machine_manager.upgrade() {
            Some(machine_manager_arc) => machine_manager_arc,
            None => {
                tracing::trace!("Setting Connected Winder | Failed to upgrade machine manager");
                return;
            }
        };
        let machine_manager_guard = block_on(machine_manager_arc.read());
        let winder2_weak = machine_manager_guard.get_machine_weak(&machine_identification_unique);
        let winder2_weak = match winder2_weak {
            Some(winder2_weak) => winder2_weak,
            None => {
                tracing::trace!("Setting Connected Winder | Failed to get machine weak");
                return;
            }
        };
        let winder2_strong = match winder2_weak.upgrade() {
            Some(winder2_strong) => winder2_strong,
            None => {
                tracing::trace!("Setting Connected Winder | Failed to upgrade to strong");
                return;
            }
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

    /// initiate connection from winder to laser
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

    /// This helper function provides an easy way
    /// to get the machine out of the Weak Reference
    ///
    /// Usage:
    ///
    ///    self.get_winder(|winder2| {
    ///        winder2.do_something     // Use the Winder here as usual
    ///    });
    fn get_winder<F, R>(&self, func: F) -> Option<R>
    where
        F: FnOnce(&mut Winder2) -> R,
    {
        self.connected_winder
            .as_ref()?
            .machine
            .upgrade()
            .map(|winder_arc| {
                let mut winder = block_on(winder_arc.lock());
                func(&mut winder)
            })
    }
}

impl LaserMachine {
    fn configure_speed_pid(&mut self, settings: PidSettings) {
        // Implement pid to control speed of winder
        let mut new_pid_settings = None;

        self.get_winder(|winder2| {
            winder2.configure_speed_pid(settings.clone());
            new_pid_settings = Some(winder2.puller_speed_controller.get_pid_params());
        });

        if let Some(params) = new_pid_settings {
            self.pid_settings = params;
        }
        // TODO: REMOVE THIS LINE
        self.pid_settings.dead = settings.dead;
        self.emit_state();
    }

    fn set_measured_diameter(&self) {
        let diameter = smol::block_on(async {
            self.laser
                .read()
                .await
                .get_data()
                .await
                .map(|laser_data| laser_data.diameter.get::<millimeter>())
        });
        self.get_winder(|winder2| {
            winder2
                .puller_speed_controller
                .set_measured_diameter(diameter.unwrap());
        });
    }
}

#[derive(Debug, Clone)]
pub struct LaserTarget {
    diameter: Length,
    lower_tolerance: Length,
    higher_tolerance: Length,
}
