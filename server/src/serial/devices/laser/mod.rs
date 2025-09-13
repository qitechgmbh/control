use std::{
    io::Write,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::machines::{MACHINE_LASER_V1, VENDOR_QITECH};
use anyhow::anyhow;
use control_core::{
    helpers::{
        hashing::{byte_folding_u16, hash_djb2},
        retry::retry_n_times,
    },
    machines::identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
        DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
    },
    modbus::{self, ModbusRequest, ModbusResponse},
    serial::{
        SerialDevice, SerialDeviceNew, SerialDeviceNewParams, panic::send_serial_device_panic,
        serial_detection::SerialDeviceRemoval,
    },
};
use serialport::SerialPort;
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, StopBits};
use smol::lock::RwLock;
use uom::si::f64::Length;

/// The struct of Laser Device
#[derive(Debug)]
pub struct Laser {
    pub data: Option<LaserData>,
    pub path: String,
}

impl SerialDevice for Laser {}

enum LaserModbusRequsts {
    ReadDiameter,
}

struct LaserDiameterResponse {
    pub diameter: Length,
}

impl TryFrom<ModbusResponse> for LaserDiameterResponse {
    type Error = anyhow::Error;

    fn try_from(value: ModbusResponse) -> Result<Self, Self::Error> {
        if value.data.len() < 3 {
            return Err(anyhow!(
                "Invalid response data length: {}",
                value.data.len()
            ));
        }
        let diameter = u16::from_be_bytes([value.data[1], value.data[2]]) as f64 / 1000.0;
        Ok(Self {
            diameter: Length::new::<uom::si::length::millimeter>(diameter),
        })
    }
}

impl From<LaserModbusRequsts> for modbus::ModbusRequest {
    fn from(request: LaserModbusRequsts) -> Self {
        match request {
            LaserModbusRequsts::ReadDiameter => Self {
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![(0 >> 8) as u8, (0 & 0xFF) as u8],
            },
        }
    }
}

impl SerialDeviceNew for Laser {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error> {
        let laser_data = Some(LaserData {
            diameter: Length::new::<uom::si::length::millimeter>(0.0),
            last_timestamp: Instant::now(),
        });
        let hash = hash_djb2(params.path.as_bytes());
        let serial = byte_folding_u16(&hash.to_le_bytes());
        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_LASER_V1,
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

        // Create a new Laser instance
        let _self = Arc::new(RwLock::new(Self {
            data: laser_data,
            path: params.path.clone(),
        }));

        // Spawn the device thread
        let device_thread_panic_tx = params.device_thread_panic_tx.clone();
        let _self_clone = _self.clone();
        let path = params.path.clone();
        thread::Builder::new()
            .name("laser".to_owned())
            .spawn(move || {
                send_serial_device_panic(path.clone(), device_thread_panic_tx.clone());
                smol::block_on(async {
                    let process_result = Self::process(_self_clone).await;

                    let removal = match process_result {
                        Ok(_) => SerialDeviceRemoval::Disconnect(path),
                        Err(e) => SerialDeviceRemoval::Error(path, e),
                    };

                    // if the task exists we want to remove the device
                    device_thread_panic_tx
                        .send(removal)
                        .await
                        .expect("Failed to send device removal signal");
                });
            })?;

        Ok((device_identification, _self))
    }
}

#[derive(Debug, Clone)]
pub struct LaserData {
    pub diameter: Length,
    pub last_timestamp: Instant,
}

impl Laser {
    pub async fn get_diameter(&self) -> Result<Length, String> {
        match &self.data {
            Some(data) => Ok(data.diameter),
            None => Err("No data from Laser".to_string()),
        }
    }

    pub async fn get_data(&self) -> Option<LaserData> {
        self.data.clone()
    }

    async fn process(_self: Arc<RwLock<Self>>) -> Result<(), anyhow::Error> {
        let path = {
            let read_guard = _self.read().await;
            read_guard.path.clone()
        };

        let request: ModbusRequest = LaserModbusRequsts::ReadDiameter.into();
        let request_buffer: Vec<u8> = request.into();

        // port configuration
        let mut port: Box<dyn SerialPort> = serialport::new(&path, 38_400)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(500)) // start with something forgiving
            .open()
            .map_err(|e| anyhow!("Failed to open port {}: {}", path, e))?;

        port.write_data_terminal_ready(true).ok();
        port.write_request_to_send(true).ok();

        port.clear(ClearBuffer::All).ok();

        loop {
            // send diameter request
            let response = retry_n_times(10, || {
                if let Err(e) = port.write_all(&request_buffer) {
                    return Err(anyhow!("Failed to write to port: {}", e));
                }

                // wait for the response
                std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                    8,
                    Duration::from_millis(10),
                    38400,
                    8,
                ));

                modbus::receive_data_modbus(&mut *port)?
                    .map(ModbusResponse::try_from)
                    .transpose()
            })?;

            if let Some(diameter_response) = response {
                // try to convert it to a LaserDiameterResponse
                let diameter_response = LaserDiameterResponse::try_from(diameter_response)?;
                // save the diameter
                let mut self_guard = _self.write().await;
                self_guard.data = Some(LaserData {
                    diameter: diameter_response.diameter,
                    last_timestamp: Instant::now(),
                });
            }
        }
    }
}
