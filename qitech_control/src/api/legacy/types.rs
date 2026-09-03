use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtherCATDeviceMetadata {
    pub configured_address: u16,
    pub name: String,
    pub vendor_id: u32,
    pub product_id: u32,
    pub revision: u32,
    pub device_identification: DeviceIdentification,
}

impl From<qitech_framework::EtherCATDeviceMetadata> for EtherCATDeviceMetadata {
    fn from(value: qitech_framework::EtherCATDeviceMetadata) -> Self {
        EtherCATDeviceMetadata {
            configured_address: value.configured_address,
            name: value.name,
            vendor_id: value.vendor_id,
            product_id: value.product_id,
            revision: value.revision,
            device_identification: DeviceIdentification {
                device_machine_identification: value.device_identification.assignment.map(|x| {
                    DeviceMachineIdentification {
                        machine_identification_unique: MachineIdentificationUnique {
                            machine_identification: MachineIdentification {
                                vendor: x.machine.machine.vendor_id,
                                machine: x.machine.machine.machine_id,
                            },
                            serial: x.machine.serial,
                        },
                        role: x.role,
                    }
                }),
                device_hardware_identification: DeviceHardwareIdentification::Ethercat(
                    DeviceHardwareIdentificationEthercat {
                        subdevice_index: match value.device_identification.hardware {
                            qitech_framework::DeviceHardwareIdentification::Ethercat(ident) => {
                                ident.subdevice_index
                            }
                            qitech_framework::DeviceHardwareIdentification::Serial(_) => {
                                unreachable!()
                            }
                        },
                    },
                ),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceIdentification {
    pub device_machine_identification: Option<DeviceMachineIdentification>,
    pub device_hardware_identification: DeviceHardwareIdentification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceMachineIdentification {
    pub machine_identification_unique: MachineIdentificationUnique,
    pub role: u16,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MachineIdentificationUnique {
    pub machine_identification: MachineIdentification,
    pub serial: u16,
}

impl From<MachineIdentificationUnique> for qitech_framework::MachineInstanceIdentification {
    fn from(id: MachineIdentificationUnique) -> Self {
        qitech_framework::MachineIdentification::new(
            id.machine_identification.vendor,
            id.machine_identification.machine,
        )
        .unique(id.serial)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MachineIdentification {
    pub vendor: u16,
    pub machine: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceHardwareIdentification {
    Ethercat(DeviceHardwareIdentificationEthercat),
    Serial(DeviceHardwareIdentificationSerial),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationEthercat {
    pub subdevice_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceHardwareIdentificationSerial {
    pub path: String,
}
