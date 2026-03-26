use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc, sync::{Arc, mpsc::Sender}, time::Duration};
use anyhow::bail;
use apis::socketio::main_namespace::{MainNamespaceEvents, machines_event::{MachineObj, MachinesEventBuilder}};
use bitvec::{order::Lsb0, slice::BitSlice};
use machine_implementations::{MachineMessage, minimal_machines::digital_input_test_machine::DigitalInputTestMachine};
use qitech_lib::{ethercat_hal::{EtherCATThreadChannel, MetaSubdevice, controller::EtherCATController, devices::{EthercatDevice, device_from_subdevice_identity_rc}, start_ethercat_thread}, machines::{Machine, MachineDataRegistry, MachineIdentificationUnique}};
pub mod apis;

/*
    This struct is only written in the main machine loop or during initialization,
    Otherwise it is simply read.
*/
pub struct SharedAppState {
    pub machines : Vec<MachineIdentificationUnique>,
    pub machines_with_channel : HashMap<MachineIdentificationUnique,Sender<MachineMessage>>,
    pub machine_subdevices : Vec<(MachineIdentificationUnique,MetaSubdevice)>    
}


impl SharedAppState {
    pub async fn send_machines_event(&self) {
        let event = MachinesEventBuilder().build(self.get_machines_meta().await);
        let main_namespace = &mut self.socketio_setup.namespaces.write().await.main_namespace;
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
    }

    pub async fn get_machines_meta(&self) -> Vec<MachineObj> {
        self.current_machines_meta.lock().await.clone()
    }

    pub async fn message_machine(
        &self,
        machine_identification_unique: &MachineIdentificationUnique,
        message: MachineMessage,
    ) -> Result<()> {
        let machines = self.api_machines.lock().await;
        let sender = machines.get(machine_identification_unique);

        if let Some(sender) = sender {
            sender.send(message).await?;
            return Ok(());
        }
        // why does a macro for return Err() exist bro ...
        bail!("Unknown machine!")
    }

    /// Removes a machine by its unique identifier
    pub async fn remove_machine(&self, machine_id: &MachineIdentificationUnique) {
        let mut current_machines = self.current_machines_meta.lock().await;
        // Retain only machines that do not match the given ID
        current_machines.retain(|m| &m.machine_identification_unique != machine_id);
        drop(current_machines);

        tracing::info!(
            "remove_machine {:?} {:?}",
            self.current_machines_meta,
            self.api_machines
        );
    }

    pub async fn add_machines_if_not_exists(&self, machines: Vec<MachineObj>) {
        let mut current_machines = self.current_machines_meta.lock().await;
        tracing::info!("add_machines_if_not_exists: {:?}", current_machines);
        // Track existing machine identifiers for quick lookup
        let existing_ids: HashSet<_> = current_machines
            .iter()
            .map(|m| m.machine_identification_unique.clone())
            .collect();

        for machine in machines {
            if !existing_ids.contains(&machine.machine_identification_unique) {
                current_machines.push(machine);
            }
        }
        drop(current_machines);

        self.send_machines_event().await;
    }

    pub async fn report_machine_error(
        &self,
        machine_identification_unique: MachineIdentificationUnique,
        error: String,
    ) {
        let mut current_machines = self.current_machines_meta.lock().await;

        for machine in current_machines.iter_mut() {
            if machine.machine_identification_unique == machine_identification_unique {
                machine.error = Some(error);
                return;
            }
        }

        current_machines.push(MachineObj {
            machine_identification_unique,
            error: Some(error),
        });
    }


    pub fn new(
    ) -> Self {
        let (socket_queue_tx, socket_queue_rx) = std::sync::mpsc::channel();
        Self {
            machines: vec![],
            machines_with_channel: HashMap::new(),
            machine_subdevices: vec![],
        }
    }
}



fn write_ecat_inputs(ecat_controller : Arc<EtherCATController>, subdevices : Vec<Rc<RefCell<dyn EthercatDevice>>>){
    assert!(ecat_controller.subdevice_count == subdevices.len());
    let inputs = ecat_controller.get_inputs();
    for i in 0..ecat_controller.subdevice_count {
        let meta_dev = ecat_controller.subdevices[i];
        let subdevice = subdevices.get(i).unwrap();
        let input_slice =  &inputs[meta_dev.start_tx..meta_dev.end_tx];
        let input_bits_slice = BitSlice::<u8, Lsb0>::from_slice(input_slice);
        {
            let mut subdevice = subdevice.borrow_mut();
            let _res = subdevice.input(input_bits_slice);
        }
    }
}

fn write_ecat_outputs(ecat_controller : Arc<EtherCATController> , subdevices : Vec<Rc<RefCell<dyn EthercatDevice>>>){
    assert!(ecat_controller.subdevice_count == subdevices.len());
    let outputs = ecat_controller.get_outputs();
    for i in 0..ecat_controller.subdevice_count {
        let meta_dev = ecat_controller.subdevices[i];
        let subdevice = subdevices.get(i).unwrap();
        let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
        let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);
        let subdevice = subdevice.borrow();
        let _res = subdevice.output(output_bits);
        ecat_controller.finish_write();
    }
}

fn main() {
    let res = start_ethercat_thread("enp101s0f4u1u2");
    let result = res.0;
    let ecat_controller = result.0;
    let ecat_channel: EtherCATThreadChannel = result.1;

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::PreOp);
    std::thread::sleep(Duration::from_millis(1000));

    let _res = ecat_channel.request_state_change(qitech_lib::ethercat_hal::EtherCATState::Op);
    std::thread::sleep(Duration::from_millis(1000));

    let mut subdevices : Vec<Rc<RefCell<dyn EthercatDevice>>> = vec![];
    let mut machine_data_reg : MachineDataRegistry = MachineDataRegistry { storage: HashMap::new() };

    for i in 0..ecat_controller.subdevice_count {
        let dev = device_from_subdevice_identity_rc(ecat_controller.subdevices[i]).unwrap();
        subdevices.push(dev.clone());       
    }    

   	let mut di_machine : DigitalInputTestMachine = DigitalInputTestMachine::new(subdevices.clone()).unwrap();
   	
    loop {
        write_ecat_inputs(ecat_controller.clone(),subdevices.clone());
        di_machine.act(Some(&mut machine_data_reg));
        di_machine.react(&machine_data_reg);
        write_ecat_outputs(ecat_controller.clone(),subdevices.clone());
    }
}
