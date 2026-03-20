use std::time::{Duration, Instant};

use anyhow::anyhow;

use stahlwerk_extension::ff01::Entry;

use crate::MachineChannel;

mod base;
use base::Base;

mod new;
mod api;

mod devices;
use devices::{Light, Scales, SignalLights};

mod tasks;
use tasks::PlateDetectTask;

mod services;
use services::WorkorderService;

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
    workorder_service: Option<WorkorderService>,

    // state
    plates_counted: u32,
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
        let instance = Self { 
            base: Base::new(channel),
            scales, 
            lights, 
            workorder_service: None, 
            plate_detect_task: PlateDetectTask::new(),
            plates_counted: 0,
        };

        // instance.enable_workorder_service();
        instance
    }

    fn update(&mut self, now: Instant) -> anyhow::Result<()> {
        
        self.scales.update();
        self.lights.update(now);
        // self.service.update(now)?;

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

        //TODO: REMOVE LATER
        let entry = Entry {
            doc_entry:     69420,
            line_number:   10,
            item_code:     "ZURO-NaN".to_string(),
            whs_code:      "01".to_string(),
            weight_bounds: stahlwerk_extension::TargetRange {
                min: 10.5,
                max: 12.5,
                desired: 11.0,
            },
        };

        /* 
        let Some(service) = &self.service else {
            // service not active
            return Ok(());
        };

        let Some(entry) = service.current_entry() else {
            // no active entry
            return Ok(());
        };
        */

        let bounds = &entry.weight_bounds;
        let expires_at = now + Self::LIGHT_EXPIRY_DURATION;

        if weight_peak < bounds.min || bounds.max < weight_peak {
            // out of bounds
            self.lights.enable_light(Light::Red, Some(expires_at));
            return Ok(());
        }

        self.lights.enable_light(Light::Green, Some(expires_at));
        self.plates_counted += 1;

        Ok(())
    }
}

// mutations
impl FF01 {
    fn disable_workorder_service(&mut self) {
        self.workorder_service = None;
    }

    fn enable_workorder_service(&mut self) {
        if self.workorder_service.is_some() {
            return;
        }

        let config_path = Self::SERVICE_CONFIG_PATH;
        let request_timeout =  Self::SERVICE_REQUEST_TIMEOUT;

        let service = match WorkorderService::new(config_path, request_timeout) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("{}", e);
                return;
            },
        };

        self.workorder_service = Some(service);
    }
}