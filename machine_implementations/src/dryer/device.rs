use super::material_presets::MaterialPreset;
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

pub const SMART_HW_ID: u16 = 4331;
const SMART_REG_HW_ID: u16 = 2000;
const SMART_REG_SW_VERSION: u16 = 2002;
const SMART_REG_TIMER_ENABLED: u16 = 2008;
const SMART_REG_TIMER_BASE: u16 = 2010;
const SMART_TIMER_ENTRY_REGS: u16 = 5;
const SMART_REG_TIMER_NEW: u16 = 2110;
const SMART_TIMER_MIN_SLOTS: u16 = 14;
const SMART_TIMER_MAX_SLOTS: u16 = 20;

const COIL_START_STOP: u16 = 272;
const COIL_SAVE_DATA: u16 = 273;
const COIL_APPLY_SETPOINT: u16 = 0x111;
const REG_TARGET_TEMP_WRITE: u16 = 0x2F;
const REG_TARGET_TEMP_READ: u16 = 0x15;
const REG_AIR_VOLUME: u16 = 0x33;
const SCHEDULE_REG_START: u16 = 0x7F;
const SCHEDULE_REG_COUNT: u16 = 28;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScheduleDay {
    /// hours * 100 + minutes, 0 = no scheduled action
    pub start_time: u16,
    pub stop_time: u16,
}

pub type WeeklySchedule = [ScheduleDay; 7]; // 0 = Mon, ..., 6 = Sun

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct SmartTimerEntry {
    pub weekly: bool,
    pub weekday: u8,
    pub hour_min: u16,
    pub year: u16,
    pub month_day: u16,
    pub enabled: bool,
    /// false = Start, true = Stop
    pub is_stop: bool,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SmartData {
    pub sw_major: u16,
    pub sw_middle: u16,
    pub sw_minor: u16,
    pub timer_enabled: bool,
    pub timer_entries: Vec<SmartTimerEntry>,
}

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
    pub target_temperature: f64,
    pub schedule: WeeklySchedule,
}

struct ActorMessage {
    request: Request<'static>,
    reply_tx: oneshot::Sender<Result<Response, anyhow::Error>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Write,
    InputRegisters,
    TargetTemp,
    Schedule,
    SmartInfo,
    SmartTimers(u16),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CycleStep {
    InputRegisters,
    TargetTemp,
    Schedule,
    SmartInfo,
    SmartTimers,
}

pub struct DryerDevice {
    pub is_smart: bool,
    pub data: Option<DryerData>,
    pub smart_data: SmartData,

    tx: mpsc::Sender<ActorMessage>,
    pending: Option<(oneshot::Receiver<Result<Response, anyhow::Error>>, PendingKind)>,
    write_queue: VecDeque<Request<'static>>,
    cycle: CycleStep,
    round_started_at: Instant,
    smart_timer_slots: u16,
    handle: JoinHandle<()>,
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
        let mut ctx = create_modbus_device_context(&meta)?;

        // Probe holding register 2000 once, synchronously, to tell V1 and Smart hardware
        // apart before the machine layer picks which machine type to construct.
        let is_smart = rt.block_on(async {
            let result = tokio::time::timeout(
                Duration::from_millis(500),
                ctx.call(Request::ReadHoldingRegisters(SMART_REG_HW_ID, 1)),
            )
            .await;
            matches!(
                result,
                Ok(Ok(Ok(Response::ReadHoldingRegisters(ref regs)))) if regs.first() == Some(&SMART_HW_ID)
            )
        });

        let (tx, rx) = mpsc::channel::<ActorMessage>(8);
        let handle = rt.spawn(run_dryer_actor(rx, ctx));

        Ok(Self {
            is_smart,
            data: None,
            smart_data: SmartData::default(),
            tx,
            pending: None,
            write_queue: VecDeque::new(),
            cycle: CycleStep::InputRegisters,
            round_started_at: Instant::now() - POLL_INTERVAL,
            smart_timer_slots: SMART_TIMER_MIN_SLOTS,
            handle,
        })
    }

    fn send_next_request(&mut self) -> Result<(), anyhow::Error> {
        if self.pending.is_some() {
            return Ok(());
        }

        if let Some(request) = self.write_queue.pop_front() {
            return self.dispatch(request, PendingKind::Write);
        }

        if self.cycle == CycleStep::InputRegisters {
            if self.round_started_at.elapsed() < POLL_INTERVAL {
                return Ok(());
            }
            self.round_started_at = Instant::now();
        }

        let (request, kind, next_step) = match self.cycle {
            CycleStep::InputRegisters => (
                Request::ReadInputRegisters(0x00, 0x21),
                PendingKind::InputRegisters,
                CycleStep::TargetTemp,
            ),
            CycleStep::TargetTemp => (
                Request::ReadInputRegisters(REG_TARGET_TEMP_READ, 1),
                PendingKind::TargetTemp,
                CycleStep::Schedule,
            ),
            CycleStep::Schedule => (
                Request::ReadInputRegisters(SCHEDULE_REG_START, SCHEDULE_REG_COUNT),
                PendingKind::Schedule,
                if self.is_smart {
                    CycleStep::SmartInfo
                } else {
                    CycleStep::InputRegisters
                },
            ),
            CycleStep::SmartInfo => (
                Request::ReadHoldingRegisters(SMART_REG_SW_VERSION, 8),
                PendingKind::SmartInfo,
                CycleStep::SmartTimers,
            ),
            CycleStep::SmartTimers => {
                let slots = self.smart_timer_slots;
                (
                    Request::ReadHoldingRegisters(SMART_REG_TIMER_BASE, slots * SMART_TIMER_ENTRY_REGS),
                    PendingKind::SmartTimers(slots),
                    CycleStep::InputRegisters,
                )
            }
        };

        self.cycle = next_step;
        self.dispatch(request, kind)
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
                // transient IO/timeout/exception: skip this tick, the next poll round retries
                tracing::debug!("dryer modbus request failed: {e}");
                return Ok(());
            }
        };

        match kind {
            PendingKind::Write => {}
            PendingKind::InputRegisters => {
                if let Response::ReadInputRegisters(regs) = response {
                    if regs.len() >= 20 {
                        let target_temperature = self
                            .data
                            .as_ref()
                            .map(|d| d.target_temperature)
                            .unwrap_or(0.0);
                        let schedule = self.data.as_ref().map(|d| d.schedule).unwrap_or_default();
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
                            target_temperature,
                            schedule,
                        });
                    }
                }
            }
            PendingKind::TargetTemp => {
                if let Response::ReadInputRegisters(regs) = response {
                    if let Some(&val) = regs.first() {
                        if val != u16::MAX {
                            if let Some(d) = &mut self.data {
                                d.target_temperature = val as f64;
                            }
                        }
                    }
                }
            }
            PendingKind::Schedule => {
                if let Response::ReadInputRegisters(regs) = response {
                    if regs.len() >= 28 {
                        let mut schedule = WeeklySchedule::default();
                        for (i, day) in schedule.iter_mut().enumerate() {
                            day.start_time = regs[i * 2];
                            day.stop_time = regs[14 + i * 2];
                        }
                        if let Some(d) = &mut self.data {
                            d.schedule = schedule;
                        }
                    }
                }
            }
            PendingKind::SmartInfo => {
                if let Response::ReadHoldingRegisters(regs) = response {
                    if regs.len() >= 8 {
                        self.smart_data.sw_major = regs[0];
                        self.smart_data.sw_middle = regs[1];
                        self.smart_data.sw_minor = regs[2];
                        self.smart_data.timer_enabled = (regs[6] & 1) != 0;
                        self.smart_timer_slots =
                            regs[7].min(SMART_TIMER_MAX_SLOTS).max(SMART_TIMER_MIN_SLOTS);
                    }
                }
            }
            PendingKind::SmartTimers(slots) => {
                if let Response::ReadHoldingRegisters(regs) = response {
                    let mut entries = Vec::new();
                    for i in 0..slots {
                        let base = (i * SMART_TIMER_ENTRY_REGS) as usize;
                        if regs.len() < base + 5 {
                            break;
                        }
                        let cycle = regs[base];
                        let flags = regs[base + 4];
                        entries.push(SmartTimerEntry {
                            weekly: (cycle & 0xFF) != 0,
                            weekday: (((cycle >> 8) & 0xFF) as u8).trailing_zeros() as u8,
                            hour_min: regs[base + 1],
                            year: regs[base + 2],
                            month_day: regs[base + 3],
                            enabled: (flags & 0xFF) != 0,
                            is_stop: ((flags >> 8) & 0xFF) != 0,
                        });
                    }
                    self.smart_data.timer_entries = entries;
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
    fn dispatch(&mut self, request: Request<'static>, kind: PendingKind) -> Result<(), anyhow::Error> {
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
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_START_STOP, true));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_START_STOP, false));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, true));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, false));
    }

    pub fn queue_set_target_temperature(&mut self, temp_celsius: f64) {
        let clamped = (temp_celsius.round() as i64).clamp(50, 180) as u16;
        self.write_queue
            .push_back(Request::WriteSingleRegister(REG_TARGET_TEMP_WRITE, clamped));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_APPLY_SETPOINT, true));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_APPLY_SETPOINT, false));
    }

    /// Queues the temperature + air-volume writes for a material preset and returns the
    /// target temperature (deg C) that was applied.
    pub fn queue_apply_material_preset(
        &mut self,
        preset: &MaterialPreset,
        throughput_kg_per_h: f64,
    ) -> u16 {
        let temp = preset.recommended_temp().clamp(50, 180);
        self.queue_set_target_temperature(temp as f64);

        let air_volume = (preset.specific_air_volume * throughput_kg_per_h).round().max(1.0) as u16;
        self.write_queue
            .push_back(Request::WriteSingleRegister(REG_AIR_VOLUME, air_volume));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, true));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, false));
        temp
    }

    pub fn queue_set_schedule(&mut self, schedule: WeeklySchedule) {
        let mut values = vec![0u16; SCHEDULE_REG_COUNT as usize];
        for (i, day) in schedule.iter().enumerate() {
            values[i * 2] = day.start_time;
            values[14 + i * 2] = day.stop_time;
        }
        self.write_queue
            .push_back(Request::WriteMultipleRegisters(SCHEDULE_REG_START, values.into()));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, true));
        self.write_queue.push_back(Request::WriteSingleCoil(COIL_SAVE_DATA, false));
    }

    pub fn queue_sync_clock(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let s = secs % 60;
        let m = (secs / 60) % 60;
        let h = (secs / 3600) % 24;
        let days_since_epoch = secs / 86400;
        let (year, month, day) = days_to_ymd(days_since_epoch);
        let hour_min = (h * 100 + m) as u16;
        let sec_day = (s * 100 + day as u64) as u16;
        let month_year = (month as u64 * 100 + year as u64 / 100) as u16;
        self.write_queue.push_back(Request::WriteMultipleRegisters(
            2005,
            vec![hour_min, sec_day, month_year].into(),
        ));
    }

    pub fn queue_set_timer_enabled(&mut self, enabled: bool) {
        self.write_queue.push_back(Request::WriteSingleRegister(
            SMART_REG_TIMER_ENABLED,
            enabled as u16,
        ));
    }

    fn timer_entry_regs(entry: &SmartTimerEntry) -> Vec<u16> {
        vec![
            (1u16 << entry.weekday) << 8 | (entry.weekly as u16),
            entry.hour_min,
            entry.year,
            entry.month_day,
            (entry.is_stop as u16) << 8 | (entry.enabled as u16),
        ]
    }

    pub fn queue_write_timer_entry(&mut self, index: u8, entry: SmartTimerEntry) {
        let base = SMART_REG_TIMER_BASE + index as u16 * SMART_TIMER_ENTRY_REGS;
        self.write_queue
            .push_back(Request::WriteMultipleRegisters(base, Self::timer_entry_regs(&entry).into()));
    }

    pub fn queue_write_new_timer_entry(&mut self, entry: SmartTimerEntry) {
        self.write_queue.push_back(Request::WriteMultipleRegisters(
            SMART_REG_TIMER_NEW,
            Self::timer_entry_regs(&entry).into(),
        ));
    }

    pub fn queue_delete_timer_entry(&mut self, index: u8) {
        let base = SMART_REG_TIMER_BASE + index as u16 * SMART_TIMER_ENTRY_REGS;
        self.write_queue
            .push_back(Request::WriteMultipleRegisters(base, vec![0u16; 5].into()));
    }
}

/// The long-running asynchronous worker loop; owns the Modbus `Context`.
async fn run_dryer_actor(mut rx: mpsc::Receiver<ActorMessage>, mut ctx: Context) {
    while let Some(msg) = rx.recv().await {
        let response_result = tokio::time::timeout(REQUEST_TIMEOUT, ctx.call(msg.request)).await;
        let result = match response_result {
            Ok(Ok(Ok(response))) => Ok(response),
            Ok(Ok(Err(exception))) => Err(anyhow!("Modbus exception: {:?}", exception)),
            Ok(Err(io_err)) => Err(anyhow!("Modbus IO error: {io_err}")),
            Err(_) => Err(anyhow!("Modbus request timed out")),
        };
        let _ = msg.reply_tx.send(result);
    }
    let _ = ctx.disconnect().await;
}

/// Civil-from-days algorithm (Howard Hinnant), days since 1970-01-01 -> (year, month, day).
fn days_to_ymd(days: u64) -> (u32, u32, u32) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z % 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}
