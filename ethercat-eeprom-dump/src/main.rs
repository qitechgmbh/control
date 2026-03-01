use std::sync::Arc;

use dump::dump_eeprom;
use ethercrab::{MainDevice, MainDeviceConfig, PduStorage, std::ethercat_now};
use read::read_eeprom;
use restore::restore_eeoprom;
use smol;

/// Maximum number of SubDevices that can be stored. This must be a power of 2 greater than 1.
const MAX_SUBDEVICES: usize = 16;
/// Maximum PDU data payload size - set this to the max PDI size or higher.
const MAX_PDU_DATA: usize = PduStorage::element_size(1024);
/// Maximum number of EtherCAT frames that can be in flight at any one time.
const MAX_FRAMES: usize = 16;
/// Maximum total PDI length.
const PDI_LEN: usize = 1024;

static PDU_STORAGE: PduStorage<MAX_FRAMES, MAX_PDU_DATA> = PduStorage::new();

pub mod cli;
pub mod dump;
pub mod ls;
pub mod print;
pub mod read;
pub mod restore;

#[tokio::main]
async fn main() {
    // cli parsing
    let matches = cli::cli().get_matches();
    let interface = matches.get_one::<String>("interface").unwrap();

    // Setup PDU
    let (tx, rx, pdu_loop) = PDU_STORAGE.try_split().expect("can only split once");

    // Setup Maindevice
    let maindevice = Arc::new(MainDevice::new(
        pdu_loop,
        Default::default(),
        MainDeviceConfig::default(),
    ));

    // Setup TX/RX task
    #[cfg(not(target_os = "windows"))]
    tokio::spawn(ethercrab::std::tx_rx_task(interface, tx, rx).expect("spawn TX/RX task"));

    // Init ethercat
    let group = maindevice
        .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
        .await
        .unwrap_or_else(|_| panic!("{}", "Failed to initalize group".to_string()));

    match matches.subcommand() {
        Some(("ls", _)) => ls::ls(group, &maindevice),
        Some(("dump", sub_matches)) => {
            let subdevice_index = sub_matches
                .get_one::<usize>("SUBDEVICE")
                .expect("subdevice index is required");
            let file = sub_matches.get_one::<String>("file");
            let result = smol::block_on(dump_eeprom(&group, &maindevice, *subdevice_index, file));
            if let Err(e) = result {
                eprintln!("Error reading EEPROM: {}", e);
            }
        }
        Some(("restore", sub_matches)) => {
            let subdevice_index = sub_matches
                .get_one::<usize>("SUBDEVICE")
                .expect("subdevice index is required");
            let file = sub_matches
                .get_one::<String>("file")
                .expect("file is required");
            let result =
                smol::block_on(restore_eeoprom(&group, &maindevice, *subdevice_index, file));
            if let Err(e) = result {
                eprintln!("Error writing EEPROM: {}", e);
            }
        }
        Some(("read", sub_matches)) => {
            let file = sub_matches
                .get_one::<String>("file")
                .expect("file is required");
            let result = smol::block_on(read_eeprom(file));
            if let Err(e) = result {
                eprintln!("Error parsing EEPROM: {}", e);
            }
        }
        _ => {}
    };
}
