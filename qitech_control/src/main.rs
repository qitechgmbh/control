use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::{Arc, OnceLock}, time::Duration};
use anyhow::bail;
use apis::socketio::main_namespace::{MainNamespaceEvents, ethercat_devices_event::EtherCatDeviceMetaData, machines_event::{MachineObj, MachinesEventBuilder}};
use bitvec::{order::Lsb0, slice::BitSlice};
use control_core::socketio::{event::GenericEvent, namespace::NamespaceCacheingLogic};
use machine_implementations::{MachineMessage, machine_identification::QiTechMachineIdentificationUnique, minimal_machines::digital_input_test_machine::DigitalInputTestMachine};
use qitech_lib::{ethercat_hal::{EtherCATThreadChannel, controller::{EtherCATAppHandle, EtherCATController}, devices::{EthercatDevice, device_from_subdevice_identity_rc}, start_ethercat_thread}, machines::{Machine, MachineDataRegistry}};
use socketioxide::{SocketIo, extract::SocketRef};
use tokio::{runtime::Runtime, sync::{RwLock, mpsc::{Receiver, Sender}}};
use crate::apis::socketio::namespaces::Namespaces;
pub mod apis;

pub struct SocketioSetup {
    pub socketio: RwLock<Option<SocketIo>>,
    pub namespaces: RwLock<Namespaces>,
    pub socket_queue_tx: Sender<(SocketRef, Arc<GenericEvent>)>,
    // Can/Should be an Arc<RefCell probably
    pub socket_queue_rx: RwLock<Receiver<(SocketRef, Arc<GenericEvent>)>>,
}

/*
    This struct is only written in the main machine loop or during initialization,
    Otherwise it is simply read.
    Except socketio
*/
pub struct SharedAppState {
    pub machines : Vec<QiTechMachineIdentificationUnique>,
    pub machines_with_channel : HashMap<QiTechMachineIdentificationUnique,Sender<MachineMessage>>,
    pub ethercat_meta_datas : Vec<EtherCatDeviceMetaData>,
    pub socketio_setup : SocketioSetup,
}


impl SharedAppState {
    pub async fn send_machines_event(&self) -> Result<(),anyhow::Error>{
        let event = MachinesEventBuilder().build(self.get_machines_meta().await);
        let mut guard =  self.socketio_setup.namespaces.write().await;
        let main_namespace = &mut guard.main_namespace;
        main_namespace.emit(MainNamespaceEvents::MachinesEvent(event));
        drop(guard);            
        Ok(())
    }

    pub async fn get_machines_meta(&self) -> Vec<MachineObj> {
        vec![]
    }

    pub async fn message_machine(
        &self,
        machine_identification_unique: &QiTechMachineIdentificationUnique,
        message: MachineMessage,
    ) -> Result<(),anyhow::Error> {
        let sender = self.machines_with_channel.get(machine_identification_unique);
        if let Some(sender) = sender {
            sender.send(message).await?;
        }
        // why does a macro for return Err() exist bro ...
        bail!("Unknown machine!")
    }

    pub fn new(
    ) -> Self {
        let (socket_queue_tx, socket_queue_rx) = tokio::sync::mpsc::channel(1024);
        Self {
            machines: vec![],
            machines_with_channel: HashMap::new(),
            socketio_setup: 
                SocketioSetup{
                socketio: RwLock::new(None),
                namespaces: RwLock::new(Namespaces::new(socket_queue_tx.clone())),
                socket_queue_tx,
                socket_queue_rx: RwLock::new(socket_queue_rx),
            },
            ethercat_meta_datas: vec![],
        }
    }
}



fn write_ecat_inputs(ecat : &mut EtherCATAppHandle,ecat_controller : Arc<EtherCATController>, subdevices : Vec<Rc<RefCell<dyn EthercatDevice>>>){
    assert!(ecat_controller.subdevice_count == subdevices.len());
    let inputs = ecat.get_inputs();
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

fn write_ecat_outputs(ecat : &mut EtherCATAppHandle, ecat_controller : Arc<EtherCATController>, subdevices : Vec<Rc<RefCell<dyn EthercatDevice>>>){
    assert!(ecat_controller.subdevice_count == subdevices.len());
    let outputs = ecat.write_outputs();
    for i in 0..ecat_controller.subdevice_count {
        let meta_dev = ecat_controller.subdevices[i];
        let subdevice = subdevices.get(i).unwrap();
        let output_slice = &mut outputs[meta_dev.start_rx..meta_dev.end_rx];
        let output_bits = BitSlice::<u8, Lsb0>::from_slice_mut(output_slice);
        let subdevice = subdevice.borrow();
        let _res = subdevice.output(output_bits);
    }
    ecat.send_outputs();
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn get_async_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio Runtime")
    })
}

fn main() {
    let rt = get_async_runtime();
    let mut state = SharedAppState::new();
    let state_arc = Arc::new(state);
    let api = rt.spawn(apis::init_api(state_arc));
    

    let res = start_ethercat_thread("enp101s0f4u1u2");
    let result = res.0;
    let ecat_controller = result.0;
    let mut ecat_handle = result.1;
    let ecat_channel: EtherCATThreadChannel = result.2;

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
        write_ecat_inputs(&mut ecat_handle,ecat_controller.clone(),subdevices.clone());
        
        di_machine.act(Some(&mut machine_data_reg));
        di_machine.react(&machine_data_reg);
        
        write_ecat_outputs(&mut ecat_handle,ecat_controller.clone(),subdevices.clone());
    }
}
