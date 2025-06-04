use std::{
    io::Write,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::{
    machines::{MACHINE_DRE, VENDOR_QITECH},
    panic::send_panic_error,
};
use anyhow::anyhow;
use control_core::{
    helpers::{
        hashing::{hashing, xor_u128_to_u16},
        retry::retry,
    },
    machines::identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
        DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
    },
    modbus::{self, ModbusRequest, ModbusResponse},
    serial::{SerialDevice, SerialDeviceNew, SerialDeviceNewParams},
};
use serial::SerialPort;
use smol::lock::RwLock;
use uom::si::f64::Length;
/// The struct of DRE Device
#[derive(Debug)]
pub struct Dre {
    pub data: Option<DreData>,
    pub path: String,
}

impl SerialDevice for Dre {}

enum DreModbusRequsts {
    ReadDiameter,
}

struct DreDiameterResponse {
    pub diameter: Length,
}

impl TryFrom<ModbusResponse> for DreDiameterResponse {
    type Error = anyhow::Error;

    fn try_from(value: ModbusResponse) -> Result<Self, Self::Error> {
        if value.data.len() < 3 {
            return Err(anyhow!(
                "Invalid response data length: {}",
                value.data.len()
            ));
        }
        let diameter = u16::from_be_bytes([value.data[1], value.data[2]]) as f64 / 1000.0;
        Ok(DreDiameterResponse {
            diameter: Length::new::<uom::si::length::millimeter>(diameter),
        })
    }
}

impl From<DreModbusRequsts> for modbus::ModbusRequest {
    fn from(request: DreModbusRequsts) -> Self {
        match request {
            DreModbusRequsts::ReadDiameter => modbus::ModbusRequest {
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![(0 >> 8) as u8, (0 & 0xFF) as u8],
            },
        }
    }
}

impl SerialDeviceNew for Dre {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Dre>>), anyhow::Error> {
        let path_clone = params.path.clone();

        let dre_data = Some(DreData {
            diameter: Length::new::<uom::si::length::millimeter>(0.0),
            last_timestamp: Instant::now(),
        });
        let hash = hashing(&params.path.clone());
        let serial = xor_u128_to_u16(hash);
        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_DRE,
                    },
                    serial: serial,
                },
                role: 0,
            }),
            device_hardware_identification: DeviceHardwareIdentification::Serial(
                DeviceHardwareIdentificationSerial {
                    path: params.path.clone(),
                },
            ),
        };

        // Create a new Dre instance
        let _self = Arc::new(RwLock::new(Dre {
            data: dre_data,
            path: params.path.clone(),
        }));

        // Spawn the device thread
        let device_thread_panix_tx = params.device_thread_panix_tx.clone();
        let _self_clone = _self.clone();
        thread::Builder::new()
            .name("DRE".to_owned())
            .spawn(move || {
                send_panic_error(path_clone.clone(), device_thread_panix_tx.clone());
                let _ = smol::block_on(async {
                    let process_result = Self::process(_self_clone).await;

                    let exit_reason = match process_result {
                        Ok(_) => anyhow!("`process` function exited normally"),
                        Err(e) => anyhow!("`process` function exited with error: {}", e),
                    };

                    // if the task exists we want to remove the device
                    device_thread_panix_tx
                        .send((path_clone, exit_reason))
                        .await
                        .expect("Failed to send device removal signal");
                });
            })?;

        Ok((device_identification, _self))
    }
}

#[derive(Debug, Clone)]
pub struct DreData {
    pub diameter: Length,
    pub last_timestamp: Instant,
}

impl Dre {
    pub async fn get_diameter(&self) -> Result<Length, String> {
        match &self.data {
            Some(data) => Ok(data.diameter),
            None => Err("No data from DRE".to_string()),
        }
    }
    pub async fn get_data(&self) -> Option<DreData> {
        self.data.clone()
    }
    async fn process(_self: Arc<RwLock<Self>>) -> Result<(), anyhow::Error> {
        let path = {
            let read_guard = _self.read().await;
            read_guard.path.clone()
        };

        let request: ModbusRequest = DreModbusRequsts::ReadDiameter.into();
        let request_buffer: Vec<u8> = request.into();

        // port configuration
        let mut port =
            serial::open(&path).map_err(|e| anyhow!("Failed to open port {}: {}", path, e))?;
        let _ = port.reconfigure(&|settings| {
            let _ = settings.set_baud_rate(serial::Baud38400);
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        });
        let _ = port.set_timeout(Duration::from_millis(100));

        loop {
            // send diameter request
            let response = retry(10, || {
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

                modbus::receive_data_modbus(&mut port)?
                    .map(ModbusResponse::try_from)
                    .transpose()
            })?;

            if let Some(diameter_response) = response {
                // try to convert it to a DreDiameterResponse
                let diameter_response = DreDiameterResponse::try_from(diameter_response)?;
                // save the diameter
                let mut self_guard = _self.write().await;
                self_guard.data = Some(DreData {
                    diameter: diameter_response.diameter,
                    last_timestamp: Instant::now(),
                });
            }
        }
    }
}
