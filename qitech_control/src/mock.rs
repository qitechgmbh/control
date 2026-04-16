use qitech_lib::ethercat_hal::{MetaSubdevice, machine_ident_read::MachineDeviceInfo};

pub struct MockEtherCatMetaData {
	pub subdevices : Vec<MetaSubdevice>,
	pub end_tx : usize,
	pub end_rx : usize
}
/*
	Still WIP! The best solution would probably be to have a program that reads ALL sdos, and either builds a .rs
	or a JSON file with the mapping, which is then used as the data source for all SDO reads, where SDO writes still wouldnt do anything, because
	the system is in an expected state already
*/

// helper functions to generate mock data
/*
	We take the end_rx and end_tx as the offset for the next call of get_____meta, so that every device has its own memory region
*/
pub fn get_aquapath_meta(starting_dev_address : u16, offset_tx : usize, offset_rx : usize) -> MockEtherCatMetaData {
	let vec = vec![	
		MetaSubdevice { name: [69, 75, 49, 49, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 72100946, revision: 1179648, vendor: 2, start_tx:offset_tx + 0, end_tx: offset_tx+0, start_rx: offset_rx+0, end_rx: offset_rx+0, device_address: starting_dev_address, initialized: true },
		MetaSubdevice { name: [69, 76, 50, 48, 48, 56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 131608658, revision: 1179648, vendor: 2, start_tx:offset_tx + 0, end_tx: offset_tx+0, start_rx: offset_rx+0, end_rx: offset_rx+1, device_address: starting_dev_address+1, initialized: true },
		MetaSubdevice { name: [69, 76, 51, 50, 48, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 209989714, revision: 1441792, vendor: 2, start_tx:offset_tx+  0, end_tx: offset_tx+16, start_rx: offset_rx+1, end_rx: offset_rx+1, device_address: starting_dev_address+2, initialized: true },
		MetaSubdevice { name: [69, 76, 53, 49, 53, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 337653842, revision: 1310720, vendor: 2, start_tx:offset_tx+ 16, end_tx:offset_tx+ 36, start_rx: offset_rx+1, end_rx: offset_rx+13, device_address: starting_dev_address+3, initialized: true },
		MetaSubdevice { name: [69, 76, 52, 48, 48, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 262287442, revision: 1310720, vendor: 2, start_tx:offset_tx+ 36, end_tx:offset_tx+ 36, start_rx: offset_rx+13, end_rx: offset_rx+17, device_address: starting_dev_address+4, initialized: true }
	];
	let subdev = vec.last().unwrap();
	MockEtherCatMetaData { 
		subdevices: vec.clone(), 
		end_tx: subdev.end_tx, 
		end_rx: subdev.end_rx 
	}
}

pub fn get_aquapath_machine_dev_infor(starting_dev_address : u16) -> Vec<MachineDeviceInfo> {
	vec![
		MachineDeviceInfo { role: 0, machine_id: 9, machine_vendor: 1, machine_serial: 13, device_address: starting_dev_address }, 
		MachineDeviceInfo { role: 1, machine_id: 9, machine_vendor: 1, machine_serial: 13, device_address: starting_dev_address+1 }, 
		MachineDeviceInfo { role: 3, machine_id: 9, machine_vendor: 1, machine_serial: 13, device_address: starting_dev_address+2 }, 
		MachineDeviceInfo { role: 4, machine_id: 9, machine_vendor: 1, machine_serial: 13, device_address: starting_dev_address+3 },
		MachineDeviceInfo { role: 2, machine_id: 9, machine_vendor: 1, machine_serial: 13, device_address: starting_dev_address+4 }
	]
}

pub fn get_extruder_meta(starting_dev_address : u16, offset_tx : usize, offset_rx : usize) -> MockEtherCatMetaData {
	let vec = vec![
		MetaSubdevice { name: [69, 75, 49, 49, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 72100946, revision: 1179648, vendor: 2, start_tx: 0, end_tx: 0, start_rx: 0, end_rx: 0, device_address: starting_dev_address, initialized: true },
		MetaSubdevice { name: [69, 76, 54, 48, 50, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 394604626, revision: 1441792, vendor: 2, start_tx: offset_tx + 0, end_tx: offset_tx + 24, start_rx: offset_rx + 0, end_rx: offset_rx + 24, device_address: starting_dev_address+1, initialized: true },
		MetaSubdevice { name: [69, 76, 50, 48, 48, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 131346514, revision: 1179648, vendor: 2, start_tx: offset_tx + 24, end_tx: offset_tx + 24, start_rx: offset_rx + 24, end_rx: offset_rx + 25, device_address: starting_dev_address+2, initialized: true },
		MetaSubdevice { name: [69, 76, 51, 48, 50, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 197996626, revision: 1310720, vendor: 2, start_tx: offset_tx + 24, end_tx: offset_tx + 28, start_rx: offset_rx + 25, end_rx: offset_rx + 25, device_address: starting_dev_address+3, initialized: true },
		MetaSubdevice { name: [69, 76, 51, 50, 48, 52, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 209989714, revision: 1441792, vendor: 2, start_tx: offset_tx + 28, end_tx: offset_tx + 44, start_rx: offset_rx + 25, end_rx: offset_rx + 25, device_address: starting_dev_address+4, initialized: true }
	];
	let subdev = vec.last().unwrap();
	MockEtherCatMetaData { 
		subdevices: vec.clone(), 
		end_tx: subdev.end_tx, 
		end_rx: subdev.end_rx 
	}
}

pub fn get_extruder_machine_dev_infor(starting_dev_address : u16) -> Vec<MachineDeviceInfo> {
	vec![
		MachineDeviceInfo { role: 0, machine_id: 22, machine_vendor: 1, machine_serial: 5, device_address: starting_dev_address   }, 
		MachineDeviceInfo { role: 1, machine_id: 22, machine_vendor: 1, machine_serial: 5, device_address: starting_dev_address+1 },
		MachineDeviceInfo { role: 2, machine_id: 22, machine_vendor: 1, machine_serial: 5, device_address: starting_dev_address+2 }, 
		MachineDeviceInfo { role: 3, machine_id: 22, machine_vendor: 1, machine_serial: 5, device_address: starting_dev_address+3 }, 
		MachineDeviceInfo { role: 4, machine_id: 22, machine_vendor: 1, machine_serial: 5, device_address: starting_dev_address+4 }		
	]
}


pub fn get_winder_meta(starting_dev_address : u16, offset_tx : usize, offset_rx : usize) -> MockEtherCatMetaData {
	let vec = vec![
		MetaSubdevice { name: [69, 75, 49, 49, 48, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 72100946, revision: 1179648, vendor: 2, start_tx: offset_tx+0, end_tx: offset_tx+0, start_rx: offset_rx+0, end_rx:offset_rx+ 0, device_address: starting_dev_address, initialized: true },
		MetaSubdevice { name: [69, 76, 50, 48, 48, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 131215442, revision: 1114112, vendor: 2, start_tx: offset_tx+0, end_tx: offset_tx+0, start_rx: offset_rx+0, end_rx:offset_rx+ 1, device_address: starting_dev_address+1, initialized: true },
		MetaSubdevice { name: [69, 76, 55, 48, 52, 49, 45, 48, 48, 53, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 461451346, revision: 1048628, vendor: 2, start_tx: offset_tx+0, end_tx: offset_tx+8, start_rx: offset_rx+1, end_rx:offset_rx+ 9, device_address: starting_dev_address+2, initialized: true },
		MetaSubdevice { name: [69, 76, 55, 48, 51, 49, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 460795986, revision: 1703936, vendor: 2, start_tx: offset_tx+8, end_tx: offset_tx+16, start_rx: offset_rx+9, end_rx: offset_rx+17, device_address: starting_dev_address+3, initialized: true },
		MetaSubdevice { name: [69, 76, 55, 48, 51, 49, 45, 48, 48, 51, 48, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], product_id: 460795986, revision: 1048606, vendor: 2, start_tx: offset_tx+16, end_tx: offset_tx+32, start_rx: offset_rx+17, end_rx:offset_rx+ 25, device_address: starting_dev_address+4, initialized: true },
	];
	let subdev = vec.last().unwrap();
	MockEtherCatMetaData { 
		subdevices: vec.clone(), 
		end_tx: subdev.end_tx, 
		end_rx: subdev.end_rx 
	}
}

pub fn get_winder_machine_dev_infor(starting_dev_address : u16) -> Vec<MachineDeviceInfo> {
	vec![
		MachineDeviceInfo { role: 0, machine_id: 2, machine_vendor: 1, machine_serial: 4, device_address: starting_dev_address }, 
		MachineDeviceInfo { role: 1, machine_id: 2, machine_vendor: 1, machine_serial: 4, device_address: starting_dev_address+1 }, 
		MachineDeviceInfo { role: 2, machine_id: 2, machine_vendor: 1, machine_serial: 4, device_address: starting_dev_address+2 }, 
		MachineDeviceInfo { role: 3, machine_id: 2, machine_vendor: 1, machine_serial: 4, device_address: starting_dev_address+3 },
		MachineDeviceInfo { role: 4, machine_id: 2, machine_vendor: 1, machine_serial: 4, device_address: starting_dev_address+4 }
	]
}
