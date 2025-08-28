use std::collections::HashMap;
use std::sync::Arc;

use smol::lock::Mutex;

use crate::machines::connection::MachineSlotGeneric;

use super::{identification::MachineIdentificationUnique, manager::MachineManager};

pub enum MachineManagerIter<'a> {
    EthercatMachines {
        iter: std::collections::hash_map::Iter<
            'a,
            MachineIdentificationUnique,
            Arc<Mutex<MachineSlotGeneric>>,
        >,
    },
    SerialMachines {
        iter: std::collections::hash_map::Iter<
            'a,
            MachineIdentificationUnique,
            Arc<Mutex<MachineSlotGeneric>>,
        >,
    },
    Done,
}

pub struct MachineManagerIterator<'a> {
    iter: MachineManagerIter<'a>,
    serial_machines: &'a HashMap<MachineIdentificationUnique, Arc<Mutex<MachineSlotGeneric>>>,
}

impl<'a> Iterator for MachineManagerIterator<'a> {
    type Item = (
        &'a MachineIdentificationUnique,
        &'a Arc<Mutex<MachineSlotGeneric>>,
    );

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.iter {
                MachineManagerIter::EthercatMachines { iter } => {
                    // Try to get the next item from ethercat_machines
                    if let Some(item) = iter.next() {
                        return Some(item);
                    } else {
                        // If ethercat_machines is exhausted, move to serial_machines
                        self.iter = MachineManagerIter::SerialMachines {
                            iter: self.serial_machines.iter(),
                        };
                        // Continue to the next iteration of the loop
                        continue;
                    }
                }
                MachineManagerIter::SerialMachines { iter } => {
                    // Return the next item from serial_machines or None if exhausted
                    if let Some(item) = iter.next() {
                        return Some(item);
                    } else {
                        // If serial_machines is also exhausted, we're done
                        self.iter = MachineManagerIter::Done;
                        return None;
                    }
                }
                MachineManagerIter::Done => {
                    // We've exhausted both collections
                    return None;
                }
            }
        }
    }
}

impl MachineManager {
    // Returns an iterator over all machines (both ethercat and serial)
    pub fn iter(&self) -> MachineManagerIterator<'_> {
        MachineManagerIterator {
            iter: MachineManagerIter::EthercatMachines {
                iter: self.ethercat_machines.iter(),
            },
            serial_machines: &self.serial_machines,
        }
    }
}
