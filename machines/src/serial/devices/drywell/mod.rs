use anyhow::anyhow;
use control_core::helpers::hashing::{byte_folding_u16, hash_djb2};
use control_core::helpers::retry::retry_n_times;
use control_core::modbus::ModbusResponse;
use control_core::modbus::{self, ModbusRequest};
use serialport::SerialPort;
use serialport::{ClearBuffer, DataBits, FlowControl, Parity, StopBits};
use smol::lock::RwLock;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crate::machine_identification::{
    DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
    DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
};
use crate::{
    MACHINE_DRYWELL_V1, SerialDevice, SerialDeviceNew, SerialDeviceNewParams, VENDOR_QITECH,
};

pub enum DrywellWriteCommand {
    WriteCoil { address: u16, value: bool },
    WriteHoldingRegister { address: u16, value: u16 },
}

#[derive(Debug, Clone)]
pub struct DrywellData {
    pub status: u16,
    pub temp_process: f64,
    pub temp_safety: f64,
    pub temp_regen_in: f64,
    pub temp_regen_out: f64,
    pub temp_fan_inlet: f64,
    pub pwm_fan1: f64,
    pub pwm_fan2: f64,
    pub temp_dew_point: f64,
    pub alarm: u16,
    pub warning: u16,
    pub temp_return_air: f64,
    pub power_process: f64,
    pub power_regen: f64,
    pub target_temperature: f64,
    pub last_timestamp: Instant,
}

#[derive(Debug)]
pub struct Drywell {
    pub data: Option<DrywellData>,
    pub path: String,
    pub shutdown_flag: Arc<AtomicBool>,
    pub cmd_sender: smol::channel::Sender<DrywellWriteCommand>,
    pub desired_setpoint: Arc<AtomicU16>,
}

impl SerialDevice for Drywell {}

enum DrywellModbusRequests {
    ReadInputRegisters,
    ReadTargetTemperature,
}

impl From<DrywellModbusRequests> for ModbusRequest {
    fn from(request: DrywellModbusRequests) -> Self {
        match request {
            DrywellModbusRequests::ReadInputRegisters => Self {
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![0x00, 0x00, 0x00, 0x21],
            },
            DrywellModbusRequests::ReadTargetTemperature => Self {
                slave_id: 1,
                function_code: modbus::ModbusFunctionCode::ReadInputRegister,
                data: vec![0x00, 0x15, 0x00, 0x01],
            },
        }
    }
}

impl SerialDeviceNew for Drywell {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error> {
        let hash = hash_djb2(params.path.as_bytes());
        let serial = byte_folding_u16(&hash.to_le_bytes());

        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_DRYWELL_V1,
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
        let desired_setpoint: Arc<AtomicU16> = AtomicU16::new(u16::MAX).into();
        let (cmd_tx, cmd_rx) = smol::channel::unbounded::<DrywellWriteCommand>();

        let _self = Arc::new(RwLock::new(Self {
            data: None,
            path: params.path.clone(),
            shutdown_flag: shutdown_flag.clone(),
            cmd_sender: cmd_tx,
            desired_setpoint: desired_setpoint.clone(),
        }));

        let _self_clone = _self.clone();
        let _ = thread::Builder::new()
            .name("drywell".to_owned())
            .spawn(move || {
                smol::block_on(async {
                    let _ = Self::process(_self_clone, cmd_rx).await;
                });
            })?;

        Ok((device_identification, _self))
    }
}

impl Drop for Drywell {
    fn drop(&mut self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }
}

impl Drywell {
    pub async fn get_data(&self) -> Option<DrywellData> {
        self.data.clone()
    }

    pub async fn set_start_stop(&self) -> Result<(), anyhow::Error> {
        self.cmd_sender
            .send(DrywellWriteCommand::WriteCoil {
                address: 272,
                value: true,
            })
            .await?;
        self.cmd_sender
            .send(DrywellWriteCommand::WriteCoil {
                address: 272,
                value: false,
            })
            .await?;
        self.cmd_sender
            .send(DrywellWriteCommand::WriteCoil {
                address: 273,
                value: true,
            })
            .await?;
        self.cmd_sender
            .send(DrywellWriteCommand::WriteCoil {
                address: 273,
                value: false,
            })
            .await?;
        Ok(())
    }

    pub async fn set_target_temperature(&self, temp_celsius: f64) -> Result<(), anyhow::Error> {
        let clamped = (temp_celsius.round() as u16).clamp(50, 180);
        self.desired_setpoint.store(clamped, Ordering::Relaxed);
        Ok(())
    }

    async fn process(
        _self: Arc<RwLock<Self>>,
        cmd_receiver: smol::channel::Receiver<DrywellWriteCommand>,
    ) -> Result<(), anyhow::Error> {
        let path = {
            let read_guard = _self.read().await;
            read_guard.path.clone()
        };

        let read_request: ModbusRequest = DrywellModbusRequests::ReadInputRegisters.into();
        let read_request_buffer: Vec<u8> = read_request.into();

        let hr_request: ModbusRequest = DrywellModbusRequests::ReadTargetTemperature.into();
        let hr_request_buffer: Vec<u8> = hr_request.into();

        let mut port: Box<dyn SerialPort> = serialport::new(&path, 57_600)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(500))
            .open()
            .map_err(|e| anyhow!("Failed to open port {}: {}", path, e))?;

        port.clear(ClearBuffer::All).ok();

        while !_self.read().await.shutdown_flag.load(Ordering::SeqCst) {
            let mut had_writes = false;
            loop {
                match cmd_receiver.try_recv() {
                    Ok(cmd) => {
                        execute_write_command(&mut port, cmd);
                        had_writes = true;
                    }
                    Err(_) => break,
                }
            }
            if had_writes {
                std::thread::sleep(Duration::from_millis(200));
            }

            let desired = _self.read().await.desired_setpoint.load(Ordering::Relaxed);
            if desired != u16::MAX {
                for buf in [
                    ModbusRequest {
                        slave_id: 1,
                        function_code: modbus::ModbusFunctionCode::PresetHoldingRegister,
                        data: vec![0x00, 0x2F, (desired >> 8) as u8, desired as u8],
                    }
                    .into(),
                    ModbusRequest {
                        slave_id: 1,
                        function_code: modbus::ModbusFunctionCode::WriteCoil,
                        data: vec![0x01, 0x11, 0xFF, 0x00],
                    }
                    .into(),
                    ModbusRequest {
                        slave_id: 1,
                        function_code: modbus::ModbusFunctionCode::WriteCoil,
                        data: vec![0x01, 0x11, 0x00, 0x00],
                    }
                    .into(),
                ] as [Vec<u8>; 3]
                {
                    port.clear(ClearBuffer::Input).ok();
                    if port.write_all(&buf).is_ok() {
                        std::thread::sleep(Duration::from_millis(50));
                        let _ = modbus::receive_data_modbus(&mut *port);
                    }
                }
                _self
                    .read()
                    .await
                    .desired_setpoint
                    .store(u16::MAX, Ordering::Relaxed);
            }

            port.clear(ClearBuffer::Input).ok();

            let response = retry_n_times(3, || {
                if let Err(e) = port.write_all(&read_request_buffer) {
                    return Err(anyhow!("Failed to write to port: {}", e));
                }
                std::thread::sleep(Duration::from_millis(100));
                modbus::receive_data_modbus(&mut *port)?
                    .map(ModbusResponse::try_from)
                    .transpose()
            });

            let prev_target = {
                let guard = _self.read().await;
                guard
                    .data
                    .as_ref()
                    .map(|d| d.target_temperature)
                    .unwrap_or(0.0)
            };
            let target_temp = {
                let mut temp = prev_target;
                if port.write_all(&hr_request_buffer).is_ok() {
                    std::thread::sleep(Duration::from_millis(100));
                    if let Ok(Some(raw)) = modbus::receive_data_modbus(&mut *port) {
                        if let Ok(hr_resp) = ModbusResponse::try_from(raw) {
                            let regs = parse_registers(&hr_resp.data);
                            if let Some(&val) = regs.first() {
                                if val != u16::MAX {
                                    temp = val as f64;
                                }
                            }
                        }
                    }
                }
                temp
            };

            if let Ok(Some(response)) = response {
                let regs = parse_registers(&response.data);
                if regs.len() >= 20 {
                    let mut guard = _self.write().await;
                    guard.data = Some(DrywellData {
                        status: regs[0],
                        temp_process: regs[1] as f64 / 10.0,
                        temp_safety: regs[2] as f64 / 10.0,
                        temp_regen_in: regs[3] as f64 / 10.0,
                        temp_regen_out: regs[4] as f64 / 10.0,
                        temp_fan_inlet: regs[5] as f64 / 10.0,
                        pwm_fan1: regs.get(6).map(|&v| v as f64).unwrap_or(0.0),
                        pwm_fan2: regs.get(7).map(|&v| v as f64).unwrap_or(0.0),
                        temp_dew_point: regs.get(12).map(|&v| v as i16 as f64 / 10.0).unwrap_or(0.0),
                        alarm: regs[14],
                        warning: regs[15],
                        temp_return_air: regs[19] as f64 / 10.0,
                        power_process: regs.get(31).map(|&v| v as f64).unwrap_or(0.0),
                        power_regen: regs.get(32).map(|&v| v as f64).unwrap_or(0.0),
                        target_temperature: target_temp,
                        last_timestamp: Instant::now(),
                    });
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }
        Ok(())
    }
}

fn execute_write_command(port: &mut Box<dyn SerialPort>, cmd: DrywellWriteCommand) {
    let request = match cmd {
        DrywellWriteCommand::WriteCoil { address, value } => ModbusRequest {
            slave_id: 1,
            function_code: modbus::ModbusFunctionCode::WriteCoil,
            data: vec![
                (address >> 8) as u8,
                address as u8,
                if value { 0xFF } else { 0x00 },
                0x00,
            ],
        },
        DrywellWriteCommand::WriteHoldingRegister { address, value } => ModbusRequest {
            slave_id: 1,
            function_code: modbus::ModbusFunctionCode::PresetHoldingRegister,
            data: vec![
                (address >> 8) as u8,
                address as u8,
                (value >> 8) as u8,
                value as u8,
            ],
        },
    };

    let buffer: Vec<u8> = request.into();
    port.clear(ClearBuffer::Input).ok();

    if port.write_all(&buffer).is_err() {
        return;
    }

    std::thread::sleep(Duration::from_millis(200));
    let _ = modbus::receive_data_modbus(&mut **port);
}

fn parse_registers(data: &[u8]) -> Vec<u16> {
    if data.is_empty() {
        return vec![];
    }
    let byte_count = data[0] as usize;
    let reg_data = &data[1..];
    if reg_data.len() < byte_count {
        return vec![];
    }
    reg_data
        .chunks(2)
        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
        .collect()
}
