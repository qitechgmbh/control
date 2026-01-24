
use anyhow::anyhow;
use modbus::{Request, request::ReadRegisters, rtu::backends::LinuxTransport};
use smol::lock::RwLock;

use control_core::{
    helpers::hashing::{byte_folding_u16, hash_djb2}, 
};
use units::{ConstZero, Frequency};

use super::US3202510;

use crate::{
    MACHINE_PELLETIZER, 
    SerialDevice, 
    SerialDeviceNew, 
    SerialDeviceNewParams, 
    VENDOR_QITECH, 
    machine_identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification, DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique
    }, serial::devices::us_3202510::{Config, ModbusInterface, RotationState, request, transport::CustomTransport}
};

use serialport::{DataBits, FlowControl, Parity, StopBits};


use std::{sync::Arc, time::Duration};

impl SerialDevice for US3202510 {}

impl SerialDeviceNew for US3202510 
{
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error> 
    { 
        let hash = hash_djb2(params.path.as_bytes());
        
        let serial = byte_folding_u16(&hash.to_le_bytes());
        
        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_PELLETIZER,
                    },
                    serial,
                },
                role: 0,
            }),
            device_hardware_identification: DeviceHardwareIdentification::Serial(
                DeviceHardwareIdentificationSerial {
                    path: params.path.clone(),
                },
            ),
        };
        
        // let interface_config = modbus_rtu::Config {
        //     slave_id:       1,
        //     path:           params.path.clone(),
        //     data_bits:      DataBits::Eight,
        //     parity:         Parity::None,
        //     stop_bits:      StopBits::One,
        //     flow_control:   FlowControl::None,
        //     timeout:        Duration::from_millis(1000),
        //     baudrate:       9600,
        //     machine_operation_delay: Duration::from_millis(100),
        // };
        
       let mut port = serialport::new(params.path.clone(), 9600).data_bits(DataBits::Eight).parity(Parity::None).open().expect("");


        let transport = CustomTransport::new(port, 1);
        
        let mut interface = ModbusInterface::new(transport);
        
        // let request: [u8; 8] = [
        //     0x01, // slave id
        //     0x04, // Read Input Registers
        //     0x00, 0x08, // start address
        //     0x00, 0x06, // quantity
        //     0xB1, 0xCA, // CRC (lo, hi)
        // ];


        let _self = Arc::new(RwLock::new(Self {
            path: params.path.clone(),
            config: Config {
                rotation_state: RotationState::Stopped,
                frequency: Frequency::ZERO,
                acceleration_level: 7,
                deceleration_level: 7,
            },
            status: None,
            failed_attempts: 0,
            interface,
        }));
        
        Ok((device_identification, _self))
    }
}