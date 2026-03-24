use anyhow::anyhow;
use control_core::helpers::hashing::{byte_folding_u16, hash_djb2};
use control_core::helpers::retry::retry_n_times;
use control_core::modbus::ModbusResponse;
use control_core::modbus::{self, ModbusRequest};
use serialport::SerialPort;
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, StopBits};
use smol::lock::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    io::Write,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use units::f64::Length;

use crate::machine_identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
    DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
};
use crate::{
    MACHINE_LASER_V1, SerialDevice, SerialDeviceNew, SerialDeviceNewParams, VENDOR_QITECH,
};
use units::length::millimeter;

/// The struct of Laser Device
#[derive(Debug)]
pub struct Laser {
    pub data: Option<LaserData>,
    pub path: String,
    pub shutdown_flag: Arc<AtomicBool>,
}

impl SerialDevice for Laser {}

enum LaserModbusRequsts {
    ReadDiameter,
}

struct LaserDiameterResponse {
    pub diameter: Length,
    pub x_axis: Option<Length>,
    pub y_axis: Option<Length>,
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
        // Depending on if its a 2 axis Laser we get more values out of the data
        let (x_axis, y_axis) = if value.data.len() >= 7 {
            let x = u16::from_be_bytes([value.data[3], value.data[4]]) as f64 / 1000.0;
            let y = u16::from_be_bytes([value.data[5], value.data[6]]) as f64 / 1000.0;
            (
                Some(Length::new::<millimeter>(x)),
                Some(Length::new::<millimeter>(y)),
            )
        } else {
            (None, None)
        };

        Ok(Self {
            diameter: Length::new::<millimeter>(diameter),
            x_axis,
            y_axis,
        })
    }
}

impl From<LaserModbusRequsts> for modbus::ModbusRequest {
    fn from(request: LaserModbusRequsts) -> Self {
        match request {
            LaserModbusRequsts::ReadDiameter => Self {
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![
                    0x00, 0x0E, // Start register = 0x000E
                    0x00, 0x03, // Read 3 registers (AvgDiameter, X, Y)
                ],
            },
        }
    }
}

impl SerialDeviceNew for Laser {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error> {
        let laser_data = Some(LaserData {
            diameter: Length::new::<millimeter>(0.0),
            x_axis: None,
            y_axis: None,
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

        let shutdown_flag: Arc<AtomicBool> = AtomicBool::new(false).into();
        // Create a new Laser instance
        let _self = Arc::new(RwLock::new(Self {
            data: laser_data,
            path: params.path.clone(),
            shutdown_flag: shutdown_flag.clone(),
        }));
        //// Spawn the device thread
        let _self_clone = _self.clone();

        let _ = thread::Builder::new()
            .name("laser".to_owned())
            .spawn(move || {
                smol::block_on(async {
                    let _ = Self::process(_self_clone).await;
                });
            })?;

        Ok((device_identification, _self))
    }
}

impl Drop for Laser {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown_flag.store(true, Ordering::SeqCst);
        println!("Laser struct dropped, thread stopped");
    }
}

#[derive(Debug, Clone)]
pub struct LaserData {
    pub diameter: Length,
    pub x_axis: Option<Length>,
    pub y_axis: Option<Length>,
    pub last_timestamp: Instant,
}

impl Laser {
    pub async fn get_diameter(&self) -> Result<Length, String> {
        match &self.data {
            Some(data) => Ok(data.diameter),
            None => Err("No data from Laser".to_string()),
        }
    }

    pub async fn get_x(&self) -> Result<Option<Length>, String> {
        match &self.data {
            Some(data) => Ok(data.x_axis),
            None => Err("No data from Laser".to_string()),
        }
    }

    pub async fn get_y(&self) -> Result<Option<Length>, String> {
        match &self.data {
            Some(data) => Ok(data.y_axis),
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

        while !_self.read().await.shutdown_flag.load(Ordering::SeqCst) {
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
                    x_axis: diameter_response.x_axis,
                    y_axis: diameter_response.y_axis,
                    last_timestamp: Instant::now(),
                });
            }
        }
        Ok(())
    }
}
