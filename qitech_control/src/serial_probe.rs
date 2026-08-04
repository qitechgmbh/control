use qitech_lib::common::get_async_runtime;
use qitech_lib::modbus::{ModbusType, Parity, SerialDeviceMeta, create_modbus_device_context};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_modbus::{Request, Response, client::Client, prelude::ReadCode};
use tokio_serial::SerialPortInfo;

const PROBE_TIMEOUT: Duration = Duration::from_millis(500);
const DRYER_SMART_HW_ID_REG: u16 = 2000;
const DRYER_SMART_HW_ID: u16 = 4331;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbedDeviceKind {
    Laser,
    Dryer { is_smart: bool },
}

#[derive(Debug, Clone)]
pub struct PortProbeResult {
    pub port_name: String,
    pub kind: ProbedDeviceKind,
}

async fn probe_is_laser(path: &str) -> bool {
    let meta = SerialDeviceMeta {
        path: path.to_owned(),
        device_name: None,
        slave_id: 1,
        baudrate: 38_400,
        bits: 8,
        stop_bits: 1,
        parity: Parity::None,
        modbus_type: ModbusType::Rtu,
    };
    let Ok(mut ctx) = create_modbus_device_context(&meta) else {
        return false;
    };
    matches!(
        tokio::time::timeout(PROBE_TIMEOUT, ctx.call(Request::ReadInputRegisters(0x0E, 3))).await,
        Ok(Ok(Ok(Response::ReadInputRegisters(ref regs)))) if regs.len() == 3
    )
}

/// Returns `Some(is_smart)` if the port responds like a dryer, `None` otherwise.
async fn probe_dryer(path: &str) -> Option<bool> {
    let meta = SerialDeviceMeta {
        path: path.to_owned(),
        device_name: None,
        slave_id: 1,
        baudrate: 57_600,
        bits: 8,
        stop_bits: 1,
        parity: Parity::None,
        modbus_type: ModbusType::Rtu,
    };
    let Ok(mut ctx) = create_modbus_device_context(&meta) else {
        return None;
    };

    let identified_by_device_id = match tokio::time::timeout(
        PROBE_TIMEOUT,
        ctx.call(Request::ReadDeviceIdentification(ReadCode::Basic, 0)),
    )
    .await
    {
        Ok(Ok(Ok(Response::ReadDeviceIdentification(resp)))) => resp
            .device_id_objects
            .iter()
            .any(|obj| obj.value_as_str().is_some_and(|s| s.contains("Dryplus"))),
        _ => false,
    };

    let identity_ok = identified_by_device_id
        || matches!(
            tokio::time::timeout(PROBE_TIMEOUT, ctx.call(Request::ReadInputRegisters(0x00, 0x21)))
                .await,
            Ok(Ok(Ok(Response::ReadInputRegisters(ref regs)))) if regs.len() == 0x21
        );
    if !identity_ok {
        return None;
    }

    let is_smart = matches!(
        tokio::time::timeout(
            PROBE_TIMEOUT,
            ctx.call(Request::ReadHoldingRegisters(DRYER_SMART_HW_ID_REG, 1)),
        )
        .await,
        Ok(Ok(Ok(Response::ReadHoldingRegisters(ref regs)))) if regs.first() == Some(&DRYER_SMART_HW_ID)
    );
    Some(is_smart)
}

/// Probes candidate CH340 ports for laser/dryer identity on a background tokio task, so the
/// real-time main loop (which also drives EtherCAT I/O) never blocks on modbus timeouts while
/// scanning for hotplugged devices. Send a fresh candidate list in; drain completed results out
/// via `tx_result`, both non-blocking from the caller's perspective.
pub fn spawn_serial_probe(
    mut rx_request: Receiver<Vec<SerialPortInfo>>,
    tx_result: Sender<Vec<PortProbeResult>>,
) {
    get_async_runtime().spawn(async move {
        while let Some(ports) = rx_request.recv().await {
            let mut results = Vec::new();
            for port in ports {
                if probe_is_laser(&port.port_name).await {
                    results.push(PortProbeResult {
                        port_name: port.port_name,
                        kind: ProbedDeviceKind::Laser,
                    });
                    continue;
                }
                if let Some(is_smart) = probe_dryer(&port.port_name).await {
                    results.push(PortProbeResult {
                        port_name: port.port_name,
                        kind: ProbedDeviceKind::Dryer { is_smart },
                    });
                }
            }
            let _ = tx_result.send(results).await;
        }
    });
}
