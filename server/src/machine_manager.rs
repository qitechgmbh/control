use std::{collections::{HashMap, HashSet}, time::{Duration, Instant}};

use machines::{Machine, MachineData, MachineSubscriptionRequest, machine_identification::MachineIdentificationUnique as MachineUID};

#[derive(Default)]
pub struct MachineManager
{
    pub machine_entries: Vec<MachineEntry>,
    pub data_registry: HashMap<MachineUID, DataEntry>,
}

impl MachineManager
{
    pub fn subscription_establish(&mut self, request: MachineSubscriptionRequest)
    {
        let entry = self.machine_entries
            .iter_mut()
            .find(|entry| entry.uid == request.subscriber)
            .expect("No such machine");

        if entry.subscriptions.insert(request.publisher)
        {
            entry.machine.subscribed_to_machine(request.publisher);
        }
    }

    pub fn subscription_terminate(&mut self, request: MachineSubscriptionRequest)
    {
        let entry = self.machine_entries
            .iter_mut()
            .find(|entry| entry.uid == request.subscriber)
            .expect("No such machine");

        if entry.subscriptions.remove(&request.publisher)
        {
            entry.machine.unsubscribed_from_machine(request.publisher);
        }
    }

    pub fn add_machines(&mut self, new_machines: Vec<Box<dyn Machine + 'static>>)
    {
        for machine in new_machines 
        {
            let uid = machine.get_machine_identification_unique();

            if self.data_registry.contains_key(&uid) {
                continue;
            }

            let data_entry = DataEntry { 
                mutation_counter: machine.mutation_counter(), 
                last_live_values: Instant::now(), 
                data: MachineData::None
            };

            let machine_entry = MachineEntry { 
                uid, 
                machine, 
                subscriptions: HashSet::new()
            };

            self.machine_entries.push(machine_entry);
            self.data_registry.insert(uid, data_entry);
        }
    }

    pub fn remove_machine(&mut self, uid: MachineUID)
    {
        self.machine_entries.retain(|entry| entry.machine.get_machine_identification_unique() != uid);
    }

    pub fn execute_machines(&mut self)
    {
        let now = Instant::now();

        // borrow 1
        for entry in self.machine_entries.iter_mut() 
        {
            // Fuck you rust - jsentity
            let (refresh_state, refresh_live_values) = {
                let data_entry = self.data_registry.get_mut(&entry.uid)
                    .expect("entries must match registry");

                // update if generation out of sync
                let refresh_state = 
                    entry.machine.mutation_counter() != data_entry.mutation_counter;

                // update once per 1/30s
                let refresh_live_values =
                    now.duration_since(data_entry.last_live_values) > Duration::from_secs_f64(1.0 / 30.0);

                if refresh_live_values {
                    data_entry.last_live_values = now;
                }

                (refresh_state, refresh_live_values)
            };

            for uid in &entry.subscriptions
            {
                let subscribed_data = self.data_registry.get_mut(uid)
                    .expect("entries must match registry");

                entry.machine.receive_machines_data(&subscribed_data.data);
            }

            entry.machine.act(now);

            // oh hey, the same fuckin ass lookup in the same function... thanks rust :D
            let data_entry = self.data_registry.get_mut(&entry.uid)
                .expect("entries must match registry");

            if refresh_state || refresh_live_values {
                entry.machine.update_machines_data(&mut data_entry.data, refresh_state, refresh_live_values);
            }
        }
    }
}

pub struct MachineEntry
{
    pub uid: MachineUID,

    pub machine: Box<dyn Machine>,

    /// other machines this one is subscribed to
    pub subscriptions: HashSet<MachineUID>,
}

pub struct DataEntry // Fuck you rust - jsentity
{
    /// generation counter of the machines state data
    /// is incremented internally in the machine when
    /// it's state is mutated. Comparing the generation
    /// reveals if we are out of sync
    pub mutation_counter: u64,

    /// tracks time since last time we checked live values
    /// to ensure we poll only as often as neded
    pub last_live_values: Instant,

    /// the data
    pub data: MachineData,
}