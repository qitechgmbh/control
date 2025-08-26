use super::{
    Machine,
    identification::{DeviceIdentification, DeviceIdentificationIdentified},
    new::MachineNewHardwareSerial,
    registry::MachineRegistry,
};
use smol::{
    channel::Sender,
    lock::{Mutex, RwLock},
};
use socketioxide::extract::SocketRef;

use crate::{
    machines::{
        connection::{MachineConnectionGeneric, MachineSlot, MachineSlotGeneric},
        identification::MachineIdentificationUnique,
        new::{MachineNewHardware, MachineNewHardwareEthercat, MachineNewParams},
    },
    serial::SerialDevice,
    socketio::event::GenericEvent,
};
use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

#[derive(Debug)]
pub struct MachineManager {
    pub ethercat_machines: HashMap<MachineIdentificationUnique, Arc<Mutex<MachineSlotGeneric>>>,
    pub serial_machines: HashMap<MachineIdentificationUnique, Arc<Mutex<MachineSlotGeneric>>>,
}

impl Default for MachineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MachineManager {
    pub fn new() -> Self {
        Self {
            ethercat_machines: HashMap::new(),
            serial_machines: HashMap::new(),
        }
    }

    pub fn set_ethercat_devices<const MAX_SUBDEVICES: usize, const MAX_PDI: usize>(
        &mut self,
        device_identifications: &Vec<DeviceIdentification>,
        machine_registry: &MachineRegistry,
        hardware: &MachineNewHardwareEthercat,
        socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
        machine_manager: Weak<RwLock<Self>>,
    ) {
        // empty ethercat machines
        self.ethercat_machines.clear();

        // group devices by machine device identification
        let device_grouping_result = group_devices_by_identification(device_identifications);

        tracing::info!(
            "Device Groups {}",
            device_grouping_result.device_groups.len()
        );

        let machine_new_hardware = MachineNewHardware::Ethercat(hardware);

        // iterate over all identified device groups but ignore unaffected machines
        for device_group in device_grouping_result.device_groups.iter() {
            // get the machine identification
            let machine_identification_unique: MachineIdentificationUnique =
                match device_group.first() {
                    Some(device_identification) => device_identification
                        .device_machine_identification
                        .machine_identification_unique
                        .clone(),
                    None => continue, // Skip this group if empty
                };

            let slot =
                self.get_or_create_slot(socket_queue_tx.clone(), machine_identification_unique);
            let mut slot = slot.lock_blocking();

            // create the machine
            let new_machine = machine_registry.new_machine(&MachineNewParams {
                device_group,
                hardware: &machine_new_hardware,
                socket_queue_tx: socket_queue_tx.clone(),
                machine_manager: machine_manager.clone(),
                namespace: slot.namespace.clone(),
            });

            slot.machine_connection = match new_machine {
                Err(err) => MachineConnectionGeneric::Error(err),
                Ok(machine) => MachineConnectionGeneric::Connected(machine),
            };
        }
    }

    fn get_or_create_slot(
        &mut self,
        socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
        machine_identification: MachineIdentificationUnique,
    ) -> Arc<Mutex<MachineSlotGeneric>> {
        if let Some(slot) = self.get(&machine_identification) {
            return slot;
        }

        let slot = Arc::new(Mutex::new(MachineSlot::new(socket_queue_tx)));
        self.ethercat_machines
            .insert(machine_identification, slot.clone());

        return slot;
    }

    pub fn get(
        &self,
        machine_identification: &MachineIdentificationUnique,
    ) -> Option<Arc<Mutex<MachineSlotGeneric>>> {
        self.ethercat_machines
            .get(machine_identification)
            .or_else(|| self.serial_machines.get(machine_identification))
            .map(|x| x.clone())
    }

    pub fn add_serial_device(
        &mut self,
        device_identification: &DeviceIdentification,
        device: Arc<RwLock<dyn SerialDevice>>,
        machine_registry: &MachineRegistry,
        socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
        machine_manager: Weak<RwLock<Self>>,
    ) {
        let hardware = MachineNewHardwareSerial { device };

        let device_identification_identified: DeviceIdentificationIdentified =
            device_identification
                .clone()
                .try_into()
                .expect("Serial devices always have machine identification");

        let machine_identification: MachineIdentificationUnique = device_identification_identified
            .device_machine_identification
            .machine_identification_unique
            .clone();

        let slot = self.get_or_create_slot(socket_queue_tx.clone(), machine_identification);
        let mut slot = slot.lock_blocking();

        let new_machine = machine_registry.new_machine(&MachineNewParams {
            device_group: &vec![device_identification_identified.clone()],
            hardware: &MachineNewHardware::Serial(&hardware),
            socket_queue_tx,
            machine_manager: machine_manager.clone(),
            namespace: slot.namespace.clone(),
        });

        slot.machine_connection = match new_machine {
            Err(err) => MachineConnectionGeneric::Error(err),
            Ok(machine) => MachineConnectionGeneric::Connected(machine),
        };

        tracing::info!("Adding serial machine {:?}", slot);
    }

    pub fn remove_serial_device(&mut self, device_identification: &DeviceIdentification) {
        let device_identification_identified: DeviceIdentificationIdentified =
            device_identification
                .clone()
                .try_into()
                .expect("Serial devices always have machine identification");

        tracing::info!(
            "Removing serial machine {:?}",
            device_identification_identified
        );

        self.serial_machines.remove(
            &device_identification_identified
                .device_machine_identification
                .machine_identification_unique,
        );
    }

    pub fn get_ethercat_weak(
        &self,
        machine_identification: &MachineIdentificationUnique,
    ) -> Option<Weak<Mutex<dyn Machine>>> {
        self.get_weak(&self.ethercat_machines, machine_identification)
    }

    pub fn get_serial_weak(
        &self,
        machine_identification: &MachineIdentificationUnique,
    ) -> Option<Weak<Mutex<dyn Machine>>> {
        self.get_weak(&self.serial_machines, machine_identification)
    }

    fn get_weak(
        &self,
        machines: &HashMap<MachineIdentificationUnique, Arc<Mutex<MachineSlotGeneric>>>,
        machine_identification: &MachineIdentificationUnique,
    ) -> Option<Weak<Mutex<dyn Machine>>> {
        let slot = machines.get(machine_identification).clone();

        return slot.and_then(|slot| {
            let connection = &slot.lock_blocking().machine_connection;
            match connection {
                MachineConnectionGeneric::Error(_) => None,
                MachineConnectionGeneric::Disconnected => None,
                MachineConnectionGeneric::Connected(machine) => Some(Arc::downgrade(&machine)),
            }
        });
    }
}

/// Groups devices by machine identification
///
/// Returns a DeviceGroupingResult containing:
/// - device_groups: a vector of devices grouped by machine identification
/// - unidentified_devices: a vector of devices that could not be identified
pub fn group_devices_by_identification(
    device_identifications: &Vec<DeviceIdentification>,
) -> DeviceGroupingResult {
    let mut device_groups: Vec<Vec<DeviceIdentificationIdentified>> = Vec::new();
    let mut unidentified_devices: Vec<DeviceIdentification> = Vec::new();

    for device_identification in device_identifications {
        // if vendor or serial or machine is 0, it is not a valid machine device
        if let Some(device_machine_identification) =
            device_identification.device_machine_identification.as_ref()
        {
            if !device_machine_identification.is_valid() {
                unidentified_devices.push(device_identification.clone());

                continue;
            }
        } else {
            unidentified_devices.push(device_identification.clone());
            continue;
        }

        // scan over all deice groups
        // get the first DeviceMachineIdentification
        // compare and append to the group
        let mut found = false;
        for check_group in device_groups.iter_mut() {
            // get first device in group
            let first_device = check_group.first().expect("group to not be empty");
            let first_device_machine_identification = &first_device
                .device_machine_identification
                .machine_identification_unique;

            // chek if it has machine identification
            if let Some(device_machine_identification) =
                device_identification.device_machine_identification.as_ref()
            {
                // compare with the current device
                if first_device_machine_identification
                    == &device_machine_identification.machine_identification_unique
                {
                    let device_identification_identified = device_identification
                        .clone()
                        .try_into()
                        .expect("should have Some(DeviceMachineIdentification)");
                    check_group.push(device_identification_identified);
                    found = true;
                    break;
                }
            }
        }

        if !found {
            let device_identification_identified = device_identification
                .clone()
                .try_into()
                .expect("should have Some(DeviceMachineIdentification)");
            device_groups.push(vec![device_identification_identified]);
        }
    }

    DeviceGroupingResult {
        device_groups,
        unidentified_devices,
    }
}

/// Structure to hold the result of grouping devices by identification
#[derive(Debug)]
pub struct DeviceGroupingResult {
    /// Devices grouped by machine identification
    pub device_groups: Vec<Vec<DeviceIdentificationIdentified>>,
    /// Devices that could not be identified
    pub unidentified_devices: Vec<DeviceIdentification>,
}
