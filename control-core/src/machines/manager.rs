use smol::lock::{Mutex, RwLock};

use crate::{
    machines::{
        identification::MachineIdentificationUnique,
        new::{MachineNewHardware, MachineNewHardwareEthercat, MachineNewParams},
    },
    serial::SerialDevice,
};
use std::{collections::HashMap, sync::Arc};

use super::{
    Machine,
    identification::{DeviceIdentification, DeviceIdentificationIdentified},
    new::MachineNewHardwareSerial,
    registry::MachineRegistry,
};
#[derive(Debug)]
pub struct MachineManager {
    pub ethercat_machines:
        HashMap<MachineIdentificationUnique, Result<Box<Mutex<dyn Machine>>, anyhow::Error>>,
    pub serial_machines:
        HashMap<MachineIdentificationUnique, Result<Box<Mutex<dyn Machine>>, anyhow::Error>>,
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
    ) {
        // empty ethercat machines
        self.ethercat_machines.clear();

        // group devices by machine device identification
        let device_grouping_result = group_devices_by_identification(device_identifications);

        log::info!(
            "[{}::set_ethercat_devices] Device Groups {:?}",
            module_path!(),
            device_grouping_result.device_groups.len()
        );

        let machine_new_hardware = MachineNewHardware::Ethercat(hardware);

        // iterate over all identified device groups but ignore unaffected machines
        for device_group in device_grouping_result.device_groups.iter() {
            // get the machine identification
            let machine_identification = match device_group.first() {
                Some(device_identification) => &device_identification.device_machine_identification,
                None => continue, // Skip this group if empty
            };

            // create the machine
            let new_machine = machine_registry.new_machine(&MachineNewParams {
                device_group,
                hardware: &machine_new_hardware,
            });

            // insert the machine into the ethercat machines map
            self.ethercat_machines.insert(
                machine_identification.machine_identification_unique.clone(),
                new_machine,
            );
        }
    }

    pub fn get(
        &self,
        machine_identification: &MachineIdentificationUnique,
    ) -> Option<&Result<Box<Mutex<dyn Machine>>, anyhow::Error>> {
        self.ethercat_machines
            .get(machine_identification)
            .or_else(|| self.serial_machines.get(machine_identification))
    }

    pub fn add_serial_device(
        &mut self,
        device_identification: &DeviceIdentification,
        device: Arc<RwLock<dyn SerialDevice>>,
        machine_registry: &MachineRegistry,
    ) {
        let hardware = MachineNewHardwareSerial { device };

        let device_identification_identified: DeviceIdentificationIdentified =
            device_identification
                .clone()
                .try_into()
                .expect("Serial devices always have machine identification");

        let new_machine = machine_registry.new_machine(&MachineNewParams {
            device_group: &vec![device_identification_identified.clone()],
            hardware: &MachineNewHardware::Serial(&hardware),
        });

        log::info!(
            "[{}::add_serial_device] Adding serial machine {:?}",
            module_path!(),
            new_machine
        );

        self.serial_machines.insert(
            device_identification_identified
                .device_machine_identification
                .machine_identification_unique,
            new_machine,
        );
    }

    pub fn remove_serial_device(&mut self, device_identification: &DeviceIdentification) {
        let device_identification_identified: DeviceIdentificationIdentified =
            device_identification
                .clone()
                .try_into()
                .expect("Serial devices always have machine identification");

        self.serial_machines.remove(
            &device_identification_identified
                .device_machine_identification
                .machine_identification_unique,
        );
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
