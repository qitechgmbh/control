use anyhow::anyhow;
use qitech_lib::common::get_async_runtime;
use qitech_lib::modbus::{
    ModbusDevice, ModbusSettings, ModbusType, Parity, SerialDeviceMeta,
    create_modbus_device_context,
};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_modbus::{
    Request, Response,
    client::{Client, Context},
};

#[derive(Debug)]
pub enum DryerDeviceError {
    IoErr(String),
    Exception(String),
    Timeout,
}

impl std::fmt::Display for DryerDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DryerDeviceError::IoErr(msg) => write!(f, "Modbus IO error: {msg}"),
            DryerDeviceError::Exception(msg) => write!(f, "Modbus exception: {msg}"),
            DryerDeviceError::Timeout => write!(f, "Modbus request timed out"),
        }
    }
}

impl std::error::Error for DryerDeviceError {}

pub fn is_running_status(status: u16) -> bool {
    status != 1 && status != 5 && status != 6
}

pub const SMART_HW_ID: u16 = 4331;
const SMART_REG_HW_ID: u16 = 2000;

const COIL_START_STOP: u16 = 272;
const COIL_SAVE_DATA: u16 = 273;
const COIL_APPLY_SETPOINT: u16 = 0x111;
const REG_TARGET_TEMP_WRITE: u16 = 0x2F;
const REG_AIR_VOLUME: u16 = 0x33;
const SCHEDULE_REG_START: u16 = 0x7F;
const SCHEDULE_REG_COUNT: u16 = 28;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
/// Number of failed identification rounds (each ~`POLL_INTERVAL`, entirely off the
/// real-time thread) before giving up and reporting "no dryer here".
const MAX_IDENTIFY_ATTEMPTS: u8 = 3;

/// `start_time`/`stop_time` are the device's native `HH*100+MM` wire encoding
/// (0 = no scheduled action) - the machine layer converts to/from minutes-since-midnight
/// for the schema, keeping this driver a dumb mirror of the actual registers.
///
/// Only ever written, never read back: `ConfigProperty` is the single source of truth for
/// what the schedule/target-temperature *should* be, so there's nothing to reconcile a
/// device echo against (unlike the old write-timestamp-suppression dance).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScheduleDay {
    pub start_time: u16,
    pub stop_time: u16,
}

pub type WeeklySchedule = [ScheduleDay; 7]; // 0 = Mon, ..., 6 = Sun

#[derive(Debug, Clone, Default)]
pub struct DryerData {
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
}

struct ActorMessage {
    request: Request<'static>,
    reply_tx: oneshot::Sender<Result<Response, anyhow::Error>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Write,
    SmartProbe,
    InputRegisters,
}

pub struct DryerDevice {
    pub is_smart: bool,
    pub data: Option<DryerData>,

    tx: mpsc::Sender<ActorMessage>,
    pending: Option<(
        oneshot::Receiver<Result<Response, anyhow::Error>>,
        PendingKind,
    )>,
    write_queue: VecDeque<Request<'static>>,
    round_started_at: Instant,
    handle: JoinHandle<()>,
    path: String,
    /// Failed identification rounds before the first successful `InputRegisters`
    /// read (`data` still `None`). Once this hits `MAX_IDENTIFY_ATTEMPTS`,
    /// `handle_response` reports a fatal error so the caller removes this device
    /// instead of polling a non-existent dryer forever.
    identify_attempts: u8,
    /// Set once, right after the first successful `InputRegisters` read, to queue a
    /// one-shot read of the Smart hardware-ID register (see `is_smart`).
    smart_probe_queued: bool,
    smart_probed: bool,
}

impl Drop for DryerDevice {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl ModbusDevice for DryerDevice {
    fn new(
        path: String,
        slave_id: u8,
        settings: Option<ModbusSettings>,
    ) -> Result<Self, anyhow::Error> {
        let meta = match settings {
            Some(s) => SerialDeviceMeta {
                path,
                device_name: None,
                slave_id,
                baudrate: s.baudrate,
                bits: s.bits,
                stop_bits: s.stop_bits,
                parity: s.parity,
                modbus_type: s.modbus_type,
            },
            None => SerialDeviceMeta {
                path,
                device_name: None,
                slave_id,
                baudrate: 57_600,
                bits: 8,
                stop_bits: 1,
                parity: Parity::None,
                modbus_type: ModbusType::Rtu,
            },
        };

        let rt = get_async_runtime();
        let _guard = rt.enter();
        let ctx = create_modbus_device_context(&meta)?;

        // Identity (is anything actually there?) and variant (V1 vs Smart, via holding
        // register 2000) are both determined below through the same non-blocking
        // request/response cycle as everything else - never by blocking here. Blocking
        // this call would stall the real-time loop that constructs this device, which
        // also drives every other machine's act() on the same thread.
        let (tx, rx) = mpsc::channel::<ActorMessage>(8);
        let handle = rt.spawn(run_dryer_actor(rx, ctx));

        Ok(Self {
            is_smart: false,
            data: None,
            tx,
            pending: None,
            write_queue: VecDeque::new(),
            round_started_at: Instant::now() - POLL_INTERVAL,
            handle,
            path: meta.path,
            identify_attempts: 0,
            smart_probe_queued: false,
            smart_probed: false,
        })
    }

    fn send_next_request(&mut self) -> Result<(), anyhow::Error> {
        if self.pending.is_some() {
            return Ok(());
        }

        if let Some(request) = self.write_queue.pop_front() {
            return self.dispatch(request, PendingKind::Write);
        }

        if self.smart_probe_queued {
            self.smart_probe_queued = false;
            return self.dispatch(
                Request::ReadHoldingRegisters(SMART_REG_HW_ID, 1),
                PendingKind::SmartProbe,
            );
        }

        if self.round_started_at.elapsed() < POLL_INTERVAL {
            return Ok(());
        }
        self.round_started_at = Instant::now();

        self.dispatch(
            Request::ReadInputRegisters(0x00, 0x21),
            PendingKind::InputRegisters,
        )
    }

    fn handle_response(&mut self) -> Result<(), anyhow::Error> {
        let is_ready = match &mut self.pending {
            Some((rx, _)) => match rx.try_recv() {
                Ok(result) => Some(result),
                Err(oneshot::error::TryRecvError::Empty) => None,
                Err(oneshot::error::TryRecvError::Closed) => {
                    return Err(anyhow!("dryer actor task died without responding"));
                }
            },
            None => return Ok(()),
        };

        let Some(result) = is_ready else {
            return Ok(());
        };
        let (_, kind) = self.pending.take().expect("pending checked above");

        let response = match result {
            Ok(response) => response,
            Err(e) => {
                // A genuine IO error means the port itself is gone (device unplugged) -
                // propagate it so the machine gets removed instead of lingering forever.
                // Timeouts/exceptions can be transient, so just skip this tick for those,
                // except while we're still identifying (data.is_none()): a bounded number
                // of failed identification rounds also means "nothing is a dryer here" and
                // should get the same treatment as an unplugged device.
                if matches!(
                    e.downcast_ref::<DryerDeviceError>(),
                    Some(DryerDeviceError::IoErr(_))
                ) {
                    return Err(e);
                }
                if kind == PendingKind::InputRegisters && self.data.is_none() {
                    self.identify_attempts += 1;
                    if self.identify_attempts >= MAX_IDENTIFY_ATTEMPTS {
                        return Err(anyhow!("no dryer responded on {}", self.path));
                    }
                }
                tracing::debug!("dryer modbus request failed: {e}");
                return Ok(());
            }
        };

        match kind {
            PendingKind::Write => {}
            PendingKind::SmartProbe => {
                if let Response::ReadHoldingRegisters(regs) = response {
                    self.is_smart = regs.first() == Some(&SMART_HW_ID);
                }
            }
            PendingKind::InputRegisters => {
                if let Response::ReadInputRegisters(regs) = response {
                    self.apply_input_registers(&regs);
                }
            }
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl DryerDevice {
    /// Parses a raw `ReadInputRegisters(0x00, 0x21)` response into `self.data`. This
    /// doubles as identification: the very first time this succeeds, it also queues the
    /// one-shot Smart-variant probe (see `smart_probe_queued`).
    fn apply_input_registers(&mut self, regs: &[u16]) {
        if regs.len() < 20 {
            return;
        }

        let was_identified = self.data.is_some();

        self.data = Some(DryerData {
            status: regs[0],
            temp_process: regs[1] as f64 / 10.0,
            temp_safety: regs[2] as f64 / 10.0,
            temp_regen_in: regs[3] as f64 / 10.0,
            temp_regen_out: regs[4] as f64 / 10.0,
            temp_fan_inlet: regs[5] as f64 / 10.0,
            pwm_fan1: regs.get(6).copied().unwrap_or(0) as f64,
            pwm_fan2: regs.get(7).copied().unwrap_or(0) as f64,
            temp_dew_point: regs.get(23).map(|&v| v as i16 as f64).unwrap_or(0.0),
            alarm: regs[14],
            warning: regs[15],
            temp_return_air: regs[19] as f64 / 10.0,
            power_process: regs.get(31).copied().unwrap_or(0) as f64,
            power_regen: regs.get(32).copied().unwrap_or(0) as f64,
        });

        self.identify_attempts = 0;
        if !was_identified && !self.smart_probed {
            self.smart_probed = true;
            self.smart_probe_queued = true;
        }
    }

    fn dispatch(
        &mut self,
        request: Request<'static>,
        kind: PendingKind,
    ) -> Result<(), anyhow::Error> {
        let (reply_tx, reply_rx) = oneshot::channel();
        match self.tx.try_send(ActorMessage { request, reply_tx }) {
            Ok(()) => {
                self.pending = Some((reply_rx, kind));
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(_)) => Ok(()),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(anyhow!("dryer actor task died")),
        }
    }

    pub fn queue_set_start_stop(&mut self) {
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_START_STOP, true));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_START_STOP, false));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, true));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, false));
    }

    pub fn queue_set_target_temperature(&mut self, temp_celsius: f64) {
        let clamped = (temp_celsius.round() as i64).clamp(50, 180) as u16;
        self.write_queue
            .push_back(Request::WriteSingleRegister(REG_TARGET_TEMP_WRITE, clamped));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_APPLY_SETPOINT, true));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_APPLY_SETPOINT, false));
    }

    pub fn queue_set_air_volume(&mut self, air_volume: u16) {
        self.write_queue
            .push_back(Request::WriteSingleRegister(REG_AIR_VOLUME, air_volume));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, true));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, false));
    }

    pub fn queue_set_schedule(&mut self, schedule: WeeklySchedule) {
        let mut values = vec![0u16; SCHEDULE_REG_COUNT as usize];
        for (i, day) in schedule.iter().enumerate() {
            values[i * 2] = day.start_time;
            values[14 + i * 2] = day.stop_time;
        }
        self.write_queue.push_back(Request::WriteMultipleRegisters(
            SCHEDULE_REG_START,
            values.into(),
        ));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, true));
        self.write_queue
            .push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, false));
    }
}

/// The long-running asynchronous worker loop; owns the Modbus `Context`.
async fn run_dryer_actor(mut rx: mpsc::Receiver<ActorMessage>, mut ctx: Context) {
    while let Some(msg) = rx.recv().await {
        let response_result = tokio::time::timeout(REQUEST_TIMEOUT, ctx.call(msg.request)).await;
        let result = match response_result {
            Ok(Ok(Ok(response))) => Ok(response),
            Ok(Ok(Err(exception))) => Err(anyhow::Error::new(DryerDeviceError::Exception(
                format!("{exception:?}"),
            ))),
            Ok(Err(io_err)) => Err(anyhow::Error::new(DryerDeviceError::IoErr(
                io_err.to_string(),
            ))),
            Err(_) => Err(anyhow::Error::new(DryerDeviceError::Timeout)),
        };
        let _ = msg.reply_tx.send(result);
    }
    let _ = ctx.disconnect().await;
}

/// Local weekday (0=Mon..6=Sun) and seconds since local midnight.
pub fn local_weekday_and_seconds() -> (u8, u32) {
    unsafe {
        let t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        // libc::tm_wday is 0=Sunday..6=Saturday; WeeklySchedule is 0=Monday..6=Sunday.
        let weekday = ((tm.tm_wday + 6) % 7) as u8;
        let seconds = (tm.tm_hour * 3600 + tm.tm_min * 60 + tm.tm_sec) as u32;
        (weekday, seconds)
    }
}
