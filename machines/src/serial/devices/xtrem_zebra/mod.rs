use anyhow::{Result, anyhow};
use control_core::helpers::hashing::{byte_folding_u16, hash_djb2};
use smol::lock::RwLock;
use std::collections::HashMap;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Ok;
use control_core::xtrem_protocol::xtrem_serial_interface::{DataAddress, Function, XtremFrame};

use crate::machine_identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
    DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
};
use crate::{
    MACHINE_XTREM_ZEBRA, SerialDevice, SerialDeviceNew, SerialDeviceNewParams, VENDOR_QITECH,
};

#[derive(Debug)]
pub struct XtremSerial {
    /// Optional cached measurement data
    pub data: Option<XtremData>,

    /// The serial/UDP “path” used to identify this device (e.g. IP, interface, etc.)
    pub path: String,

    /// Flag used by the background thread to know when to shut down
    pub shutdown_flag: Arc<AtomicBool>,
}

impl SerialDevice for XtremSerial {}

struct XtremResponse {
    pub raw: Vec<u8>,
}

impl TryFrom<XtremResponse> for XtremFrame {
    type Error = anyhow::Error;

    fn try_from(value: XtremResponse) -> Result<Self, Self::Error> {
        let data = value.raw;

        if data.len() < 14 {
            return Err(anyhow!("Invalid Xtrem message length: {}", data.len()));
        }

        let stx = data[0];
        let id_origin = data[1];
        let id_dest = data[3];
        let function_char = data[5] as char;
        let function = Function::from_char(function_char)
            .ok_or_else(|| anyhow! {"Invalid function character: {}", function_char})?;

        let data_address = u16::from_be_bytes([data[6], data[7]]);
        let data_length = data[10];

        let dl_value = data_length as usize;
        if data.len() < 12 + dl_value + 3 {
            return Err(anyhow!(
                "Incomplete data section: expected {} bytes of DATA, got {}",
                dl_value,
                data.len() - 12
            ));
        }

        let data_start = 12;
        let data_end = 12 + dl_value;
        let payload = data[data_start..data_end].to_vec();

        let lrc = data[data_end];
        let etx = data[data_end + 1];

        Ok(Self {
            stx,
            id_origin,
            id_dest,
            function,
            data_address,
            data_length,
            data: payload,
            lrc,
            etx,
        })
    }
}

#[derive(Debug, Clone)]
struct XtremRequest {
    pub id_origin: u8,
    pub id_dest: u8,
    pub data_address: DataAddress,
    pub function: Function,
    pub data: Vec<u8>,
}

impl From<XtremRequest> for XtremFrame {
    fn from(request: XtremRequest) -> Self {
        let id_origin = request.id_origin;
        let id_dest = request.id_dest;
        let data_address = request.data_address.as_hex();
        let data_length = request.data.len() as u8;

        // Build frame body (everything between STX and ETX)
        let mut frame_body = Vec::new();
        frame_body.push(id_origin);
        frame_body.push(id_dest);
        frame_body.push(request.function.as_char() as u8);
        frame_body.extend_from_slice(&data_address.to_be_bytes());
        frame_body.push(data_length);
        frame_body.extend_from_slice(&request.data);

        let lrc = XtremFrame::compute_lrc(&frame_body);

        Self {
            stx: 0x02,
            id_origin,
            id_dest,
            function: request.function,
            data_address,
            data_length,
            data: request.data,
            lrc,
            etx: 0x03,
        }
    }
}

impl SerialDeviceNew for XtremSerial {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>)> {
        let xtrem_data = Some(XtremData {
            current_weight: 0.0,
            last_timestamp: Instant::now(),
        });

        let hash = hash_djb2(params.path.as_bytes());
        let serial = byte_folding_u16(&hash.to_le_bytes());
        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_XTREM_ZEBRA,
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

        let shutdown_flag: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

        let _self = Arc::new(RwLock::new(Self {
            data: xtrem_data,
            path: params.path.clone(),
            shutdown_flag: shutdown_flag.clone(),
        }));

        let _self_clone = _self.clone();
        let path = params.path.clone();

        // Spawn the XTREM communication thread
        thread::Builder::new()
            .name("xtrem_zebra".to_owned())
            .spawn(move || {
                smol::block_on(async move {
                    if let Err(e) = Self::process_udp(_self_clone, path, shutdown_flag).await {
                        eprintln!("[XTREM] Error: {:?}", e);
                    }
                });
            })?;

        Ok((device_identification, _self))
    }
}

impl Drop for XtremSerial {
    fn drop(&mut self) {
        // Signal shutdown
        self.shutdown_flag.store(true, Ordering::SeqCst);
        println!("Laser struct dropped, thread stopped");
    }
}

impl XtremSerial {
    pub async fn get_data(&self) -> Option<XtremData> {
        self.data.clone()
    }
    /// Asynchronous UDP communication handler for the XTREM Zebra device.
    async fn process_udp(
        this: Arc<RwLock<Self>>,
        _path: String,
        shutdown: Arc<AtomicBool>,
    ) -> std::result::Result<(), anyhow::Error> {
        let rx_port = 5555; // Device -> PC
        let tx_addr = "192.168.4.255:4444"; // PC -> Broadcast domain (we send unicast by ID)

        let sock_rx = UdpSocket::bind(("0.0.0.0", rx_port))?;
        sock_rx.set_nonblocking(true)?;

        let sock_tx = UdpSocket::bind("0.0.0.0:0")?;
        sock_tx.set_broadcast(true)?;
        sock_tx.connect(tx_addr)?;

        println!(
            "[XTREM] Listening on UDP {} / sending to {}",
            rx_port, tx_addr
        );

        // Function to build a request for a specific destination ID
        let build_request = |dest_id: u8| -> Vec<u8> {
            let request = XtremRequest {
                id_origin: 0x00,
                id_dest: dest_id,
                data_address: DataAddress::Weight,
                function: Function::ReadRequest,
                data: Vec::new(),
            };
            let frame: XtremFrame = request.into();
            frame.as_bytes()
        };

        // Known device IDs
        let device_ids = [0x01, 0x02];
        let cmds: Vec<Vec<u8>> = device_ids.iter().map(|&id| build_request(id)).collect();

        while !shutdown.load(Ordering::Relaxed) {
            // Send requests to all known devices
            for (i, cmd) in cmds.iter().enumerate() {
                let dest_id = device_ids[i];
                println!("[XTREM] Sending request to device {:02X}", dest_id);
                sock_tx.send(cmd)?;
                thread::sleep(Duration::from_millis(10));
            }

            let start = Instant::now();
            let timeout = Duration::from_millis(300);
            let mut buf = [0u8; 2048];
            let mut total_weight = 0.0;
            let mut received_count = 0;

            // Wait for replies from both scales
            while start.elapsed() < timeout && received_count < device_ids.len() {
                match sock_rx.recv(&mut buf) {
                    std::result::Result::Ok(n) => {
                        let ascii = String::from_utf8_lossy(&buf[..n]);
                        println!("[XTREM] RX {} bytes: {}", n, ascii);

                        if let std::result::Result::Ok(frame) =
                            XtremFrame::try_from(XtremResponse {
                                raw: buf[..n].to_vec(),
                            })
                        {
                            let weight = XtremFrame::parse_weight_from_response(&buf[..n]);
                            let id = frame.id_origin;

                            println!("[XTREM] Device {:02X} weight: {:.3} kg", id, weight);
                            total_weight += weight;
                            received_count += 1;
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("[XTREM] Socket error: {:?}", e);
                        break;
                    }
                }
            }

            // Update the shared structure with the combined weight
            {
                let mut device = this.write().await;
                device.data = Some(XtremData {
                    current_weight: total_weight,
                    last_timestamp: Instant::now(),
                });
            }

            println!(
                "[XTREM] Combined total weight from {} devices: {:.3} kg",
                received_count, total_weight
            );

            thread::sleep(Duration::from_millis(300));
        }

        println!("[XTREM] Shutdown signal received, stopping thread.");
        std::result::Result::Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct XtremData {
    pub current_weight: f64,
    pub last_timestamp: Instant,
}
