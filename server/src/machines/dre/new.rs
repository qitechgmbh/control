use std::{io::Write, sync::Arc, thread, time::Duration};

use super::Dre;
use anyhow::{Error,anyhow};
use control_core::{modbus, serial::SerialNew};
use serial::SerialPort;
use smol::lock::RwLock;

impl SerialNew for Dre {
    fn new_serial(path: &str) -> Result<Self, Error> {
        let path_string = path.to_string();
        let diameter = Arc::new(RwLock::new(Err(anyhow::anyhow!("No connection"))));
        let failed_request_counter = Arc::new(RwLock::new(0));

        let diameter_clone = Arc::clone(&diameter);
        let counter_clone = Arc::clone(&failed_request_counter);
        let path_clone = path_string.clone();

        thread::Builder::new()
            .name("DRE".to_owned())
            .spawn(move || {
                smol::block_on(async move {
                    loop {
                        let request = modbus::ModbusRequest {
                            slave_id: 1,
                            function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                            data: vec![(0 >> 8) as u8, (0 & 0xFF) as u8],
                        };
                        let request_vec: Vec<u8> = request.into();

                        match serial::open(&path_clone) {
                            Ok(mut port) => {
                                let _ = port.reconfigure(&|settings| {
                                    let _ = settings.set_baud_rate(serial::Baud38400);
                                    settings.set_char_size(serial::Bits8);
                                    settings.set_parity(serial::ParityNone);
                                    settings.set_stop_bits(serial::Stop1);
                                    settings.set_flow_control(serial::FlowNone);
                                    Ok(())
                                });
                                let _ = port.set_timeout(Duration::from_millis(100));

                                if let Err(e) = port.write_all(&request_vec) {
                                    eprintln!("Failed to write to port: {}", e);
                                    continue;
                                }

                                std::thread::sleep(modbus::calculate_modbus_rtu_timeout(
                                    8,
                                    Duration::from_millis(10),
                                    38400,
                                    8,
                                ));
                                let result = modbus::receive_data_modbus(&mut port);
                                if let Some(value) = result {
                                    let val = u16::from_be_bytes([value[0], value[1]]) as f32 / 1000.0;
                                    {
                                        let mut diam = diameter_clone.write().await;
                                        *diam = Ok(val);
                                    }
                                    {
                                        let mut count = counter_clone.write().await;
                                        *count = 0;
                                    }
                                } else {
                                    let mut count = counter_clone.write().await;
                                    *count += 1;
                                    if *count > 10 {
                                        let mut diam = diameter_clone.write().await;
                                        *diam = Err(anyhow!("Failed to read from serial device after 10 tries"));
                                        drop(port);
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to open port: {}", e);
                                break;
                            }
                        }
                    }
                });
            })
            .expect("Failed to spawn DRE update thread");

        Ok(Dre {
            diameter,
            path: path_string,
        })
    }
}
