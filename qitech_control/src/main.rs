use anyhow::Result;
use apis::socketio::queue::start_socketio_queue;
use app_state::SharedAppState;
use machine_implementations::laser::LaserMachine;
use machine_implementations::machine_identification::QiTechMachineIdentificationUnique;
use machine_implementations::registry::MACHINE_REGISTRY;
use machine_implementations::{
    Hardware, IdentifiedEthercat, IdentifiedModbus, MachineHardware, QiTechMachine,
};
use qitech_lib::ethercat_hal::machine_ident_read::MachineDeviceInfo;
use qitech_lib::ethercat_hal::{EtherCATState, MetaSubdevice};
use qitech_lib::machines::MachineDataRegistry;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use qitech_lib::modbus::clients::example_client::ExampleClient;
use qitech_lib::modbus::devices::qitech_laser::LaserDevice;
use qitech_lib::modbus::managers::ExampleDeviceManager;
use qitech_lib::modbus::managers::example_manager::ExampleScheduler;
use qitech_lib::modbus::start_modbus_async_task;

use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, OnceLock},
    time::Duration,
};

use tokio::runtime::Runtime;

use crate::ethercat::{EtherCAT, EtherCATDevice};

pub mod apis;
mod app_state;
mod ethercat;

static RUNTIME: OnceLock<Runtime> = OnceLock::new();
fn get_async_runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create Tokio Runtime")
    })
}

struct Control {
    pub ethercat: EtherCAT,
    pub runtime: &'static Runtime,
    pub shared_state: Arc<SharedAppState>,
    pub subdevices: Vec<EtherCATDevice>,
    pub hardware: HashMap<MachineIdentificationUnique, MachineHardware>,
    pub machines: Vec<Box<dyn QiTechMachine>>,
    pub machine_errors: HashMap<MachineIdentificationUnique, String>,
    pub machine_data_reg: MachineDataRegistry,
    pub modbus_mgrs: Vec<Rc<RefCell<ExampleDeviceManager>>>,
}

impl Control {
    pub fn new() -> Self {
        let machines = vec![];
        let machine_data_reg = MachineDataRegistry {
            storage: HashMap::new(),
        };
        Control {
            ethercat: EtherCAT::default(),
            runtime: get_async_runtime(),
            shared_state: Arc::new(SharedAppState::new()),
            machines,
            machine_data_reg,
            subdevices: vec![],
            modbus_mgrs: vec![],
            hardware: HashMap::new(),
            machine_errors: HashMap::new(),
        }
    }

    pub fn generate_machine_hardware_from_serial(
        &mut self,
        mgr: Rc<RefCell<ExampleDeviceManager>>,
    ) {
        let laser_device: Rc<RefCell<LaserDevice<ExampleScheduler>>> =
            ExampleDeviceManager::register_device(mgr.clone(), 1);
        let id_modbus: IdentifiedModbus = IdentifiedModbus {
            hw: laser_device,
            manager: mgr.clone(),
        };
        let ident = MachineIdentificationUnique {
            machine_ident: LaserMachine::MACHINE_IDENTIFICATION,
            serial: 1,
        };
        let mut hw = MachineHardware {
            hw: vec![],
            identification: ident,
            ethercat_interface: None,
        };
        hw.hw.push(Hardware::Modbus(id_modbus));
        self.modbus_mgrs.push(mgr.clone());
        self.hardware.insert(ident, hw);
    }

    pub fn run_main_loop(&mut self) {
        let shared_state = self.shared_state.clone();
        let has_ethercat = self.ethercat.is_online();

        log_error(self.ethercat.goto_state(EtherCATState::Op));

        self.runtime.spawn(async move {
            if has_ethercat {
                shared_state.send_ethercat_setup_done().await;
            }

            shared_state.send_machines_event().await;
        });

        loop {
            if self.ethercat.is_online() {
                // We check to prevent console spam
                log_error(self.ethercat.write_inputs(&mut self.subdevices));
            }

            self.run_machines();

            if self.ethercat.is_online() {
                // We check to prevent console spam
                log_error(self.ethercat.write_outputs(&mut self.subdevices));
            }

            std::thread::sleep(Duration::from_micros(100)); // TODO: do we want this sleep, can we not
            // wait for an event/response to happen
        }
    }

    fn run_machines(&mut self) {
        let reg = &mut self.machine_data_reg;

        for machine in self.machines.iter_mut() {
            reg.zero_entry(machine.get_identification());
            machine.act(Some(reg));
        }

        for machine in self.machines.iter_mut() {
            machine.react(reg);
        }
    }

    fn start_serial(&mut self) {
        let (tx, rx) = ExampleClient::create_channels(); // TODO: need better name
        let modbus_task = start_modbus_async_task("/dev/ttyUSB0".to_string(), 1, 38400, rx);
        self.runtime.spawn(modbus_task);
        let modbus_mgr = ExampleDeviceManager::new(tx);
        self.generate_machine_hardware_from_serial(modbus_mgr);
    }

    fn create_hardware_from_ethercat(&mut self) -> Result<()> {
        let mut devices = self.ethercat.create_devices()?;

        for (_, dev) in devices.iter() {
            self.subdevices.push(dev.clone());
        }

        // let guessed_mappings = guess_mappings_by_device_sequence(devices); // we could do this if
        // we want to. We would need to write a lot of lambdas tho.

        // TODO: These mapping are what we want in the future
        // let saved_mapping = read_saved_device_mappings();
        // self.generate_ethercat_hardware_from_mapping(saved_mapping, devices);

        let eeprom_mappings = self.ethercat.read_device_identification_from_eeprom()?;
        self.generate_ethercat_hardware_from_mapping(eeprom_mappings, devices);

        // self.generate_ethercat_hardware_from_mapping(guessed_mappings, devices);

        // TODO: should not need controller
        // self.shared_state.fill_ethercat_metadata(ecat_controller.clone(), eeprom_mappings);

        println!(
            "Initialized {} machine hardware from EtherCAT",
            self.hardware.keys().len()
        );
        Ok(())
    }

    pub fn create_machines_from_hardware(&mut self) {
        for key in self.hardware.keys() {
            println!("{:?}", key);
            let result = MACHINE_REGISTRY.new_machine(*key, self.hardware[key].clone());
            match result {
                Ok(machine) => {
                    let _res = self.shared_state.add_machine_sync(
                        QiTechMachineIdentificationUnique::from(*key),
                        None,
                        Some(machine.get_api_sender()),
                    );
                    self.machines.push(machine);
                }
                Err(e) => {
                    println!("{:?}", e);
                    self.machine_errors.insert(*key, e.to_string());
                }
            };
        }
    }

    // TODO: remove devices that have been assigned, so there not mapped again.
    pub fn generate_ethercat_hardware_from_mapping(
        &mut self,
        device_infos: Vec<MachineDeviceInfo>,
        mut ethercat_devices_mapped: Vec<(MetaSubdevice, EtherCATDevice)>,
    ) {
        let mut hw_map: HashMap<u16, (MetaSubdevice, EtherCATDevice)> = ethercat_devices_mapped
            .drain(..)
            .map(|(meta, device)| (meta.device_address, (meta, device)))
            .collect();

        let mut combined_list: Vec<(MachineDeviceInfo, MetaSubdevice, EtherCATDevice)> =
            device_infos
                .into_iter()
                .filter_map(|info| {
                    let address = info.device_address;
                    hw_map.remove(&address).map(|hw| {
                        (info, hw.0, hw.1) // This is a 3-tuple
                    })
                })
                .collect();

        combined_list.sort_by_key(|f| (f.0.machine_id, f.0.machine_serial));
        for (dev_info, _, eth) in combined_list.drain(0..combined_list.len()) {
            let identification = MachineIdentificationUnique {
                machine_ident: MachineIdentification {
                    vendor: dev_info.machine_vendor,
                    machine: dev_info.machine_id,
                },
                serial: dev_info.machine_serial as u32,
            };
            if self.hardware.get(&identification).is_none() {
                self.hardware.insert(
                    identification,
                    MachineHardware {
                        hw: vec![],
                        identification,
                        ethercat_interface: self.ethercat.get_channel(), // TODO: why is it called
                                                                         // interface?
                    },
                );
            }
            let ethercat_hw = Hardware::Ethercat(IdentifiedEthercat {
                hw: eth,
                ident: dev_info,
            });
            let hw = self.hardware.get_mut(&identification).unwrap();
            hw.hw.push(ethercat_hw);
        }
    }
}

fn log_error<T>(res: Result<T>) {
    if let Err(e) = res {
        tracing::error!("{e}");
    }
}

fn mock_logic() {}

fn main_logic() {
    let mut control = Control::new();
    let rt = control.runtime;
    let shared_state = control.shared_state.clone();

    rt.spawn(apis::init_api(shared_state.clone())); // TODO: how to handle when the API crashes?
    rt.spawn(start_socketio_queue(shared_state));

    if let Err(e) = control.ethercat.init("eth0") {
        tracing::error!("{e}");
    }

    std::thread::sleep(Duration::from_millis(5000)); // TODO: wait for responce instead of sleep

    control.start_serial();
    log_error(control.create_hardware_from_ethercat());
    // Create more machines...

    control.create_machines_from_hardware();
    control.run_main_loop();
}

fn main() {
    #[cfg(feature = "mock")]
    mock_logic();

    #[cfg(not(feature = "mock"))]
    main_logic();
}
