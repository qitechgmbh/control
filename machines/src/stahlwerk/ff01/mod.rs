use std::time::{Duration, Instant};

use anyhow::anyhow;

use crate::MachineChannel;

mod base;
use base::Base;

mod new;
mod api;

mod devices;
use devices::{Light, Scales, SignalLights};

mod tasks;
use tasks::PlateDetectTask;

mod workorder_service;
use workorder_service::WorkorderService;

#[derive(Debug)]
pub struct FF01 {
    // common machine stuff
    base: Base,

    // devices
    scales: Scales,
    lights: SignalLights,

    // tasks
    plate_detect_task: PlateDetectTask,

    // services
    workorder_service: WorkorderService,

    // state
    plate_count: u32,
}

// constants
impl FF01 {
    const LIGHT_EXPIRY_DURATION: Duration = Duration::from_millis(500);

    const SERVICE_CONFIG_PATH: &str = "/home/qitech/config.json";
    const SERVICE_REQUEST_TIMEOUT: Duration = Duration::from_millis(1000);
}

// public interface
impl FF01 {
    pub fn new(channel: MachineChannel, scales: Scales, lights: SignalLights) -> Self {
        let mut instance = Self { 
            base: Base::new(channel),
            scales, 
            lights, 
            workorder_service: WorkorderService::new(Self::SERVICE_REQUEST_TIMEOUT), 
            plate_detect_task: PlateDetectTask::new(),
            plate_count: 0,
        };

        instance.workorder_service.set_enabled(true);
        if let Err(e) = instance.workorder_service.connect(Self::SERVICE_CONFIG_PATH) {
            tracing::error!("Failed to connect client: {}", e);
        }
        instance
    }

    fn update(&mut self, now: Instant) -> anyhow::Result<()>  {
        self.scales.update();
        self.lights.update(now);
        // check for response to update entry
        self.workorder_service.update_recv()?;
        self.update_plate_count(now)?;
        // send with updated plate count
        self.workorder_service.update_send(now, self.plate_count)?;
        Ok(())
    }

    fn update_plate_count(&mut self, now: Instant) -> anyhow::Result<()> {
        let Some(weight) = self.scales.weight() else {
            // received no data from scales
            self.plate_detect_task.reset();
            return Ok(());
        };

        if !self.plate_detect_task.check(weight) {
            // no plate detected
            return Ok(());
        }

        let Some(weight_peak) = self.scales.weight_peak() else {
            // unreachable
            return Err(anyhow!("No weight_peak but should have"));
        };

        // current peak is the plate measurement, so reset it
        self.scales.weight_peak_reset();

        let Some(entry) = self.workorder_service.current_entry() else {
            // no active entry
            return Ok(());
        };

        let bounds = &entry.weight_bounds;
        let expires_at = now + Self::LIGHT_EXPIRY_DURATION;

        if weight_peak < bounds.min || bounds.max < weight_peak {
            // out of bounds
            self.lights.enable_light(Light::Red, Some(expires_at));
            return Ok(());
        }

        self.lights.enable_light(Light::Green, Some(expires_at));
        self.plate_count += 1;
        Ok(())
    }
}