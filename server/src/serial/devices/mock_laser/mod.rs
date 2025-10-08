use std::sync::Arc;
use std::time::Instant;

use control_core::{
    helpers::hashing::{byte_folding_u16, hash_djb2},
    machines::identification::{
        DeviceHardwareIdentification, DeviceHardwareIdentificationSerial, DeviceIdentification,
        DeviceMachineIdentification, MachineIdentification, MachineIdentificationUnique,
    },
    serial::{SerialDevice, SerialDeviceNew, SerialDeviceNewParams},
};
use smol::lock::RwLock;
use uom::si::{f64::Length, length::millimeter};

use crate::machines::{MACHINE_LASER_V1, VENDOR_QITECH};

/// Mock laser serial device for testing LaserMachine
/// This provides a realistic laser data simulation with diameter fluctuations
#[derive(Debug)]
pub struct MockLaserDevice {
    pub path: String,
    pub data: Option<MockLaserData>,
    pub start_time: Instant,
}

#[derive(Debug, Clone)]
pub struct MockLaserData {
    pub diameter: Length,
    pub x_axis: Option<Length>,
    pub y_axis: Option<Length>,
    pub last_timestamp: Instant,
}

impl SerialDevice for MockLaserDevice {}

impl SerialDeviceNew for MockLaserDevice {
    fn new_serial(
        params: &SerialDeviceNewParams,
    ) -> Result<(DeviceIdentification, Arc<RwLock<Self>>), anyhow::Error>
    where
        Self: Sized,
    {
        // Generate a unique serial number based on the path
        let hash = hash_djb2(params.path.as_bytes());
        let serial = byte_folding_u16(&hash.to_le_bytes());

        let device_identification = DeviceIdentification {
            device_machine_identification: Some(DeviceMachineIdentification {
                machine_identification_unique: MachineIdentificationUnique {
                    machine_identification: MachineIdentification {
                        vendor: VENDOR_QITECH,
                        machine: MACHINE_LASER_V1, // Same ID as real laser
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

        let now = Instant::now();
        let mock_laser_device = Arc::new(RwLock::new(MockLaserDevice {
            path: params.path.clone(),
            data: Some(MockLaserData {
                diameter: Length::new::<millimeter>(1.75),
                x_axis: Some(Length::new::<millimeter>(1.75)),
                y_axis: Some(Length::new::<millimeter>(1.75)),
                last_timestamp: now,
            }),
            start_time: now,
        }));

        Ok((device_identification, mock_laser_device))
    }
}

impl MockLaserDevice {
    pub async fn get_data(&self) -> Option<MockLaserData> {
        self.data.clone()
    }

    /// Update the mock laser data with realistic fluctuations
    /// - Base diameter around 1.75mm
    /// - Normal fluctuations of +/- 0.5mm with smooth transitions
    /// - Random spikes every ~60 seconds to >3mm or <1mm
    pub fn update_data(&mut self) {
        let now = Instant::now();
        let elapsed_secs = now.duration_since(self.start_time).as_secs_f64();
        
        // Base diameter
        let base_diameter = 1.7;
        
        // Slow wave component (smooth up and down over time)
        // Period of about 60 seconds - slower changes
        // Keep amplitude small so it mostly stays around 1.75mm
        let slow_wave = 0.15 * (elapsed_secs / 30.0).sin();
        
        // Medium wave component (adds more variation)
        // Period of about 20 seconds
        let medium_wave = 0.08 * (elapsed_secs / 10.0).sin();
        
        // Fast wave component (small rapid fluctuations)
        // Period of about 3 seconds
        let fast_wave = 0.05 * (elapsed_secs / 1.5).sin();
        
        // Random spike component
        // Use a combination of sine waves to create periodic but somewhat random spikes
        // This creates spikes roughly every minute but with some variation
        let spike_trigger = ((elapsed_secs / 60.0).sin() * 10.0).sin();
        let spike_magnitude = if spike_trigger > 0.99 {
            // Spike up to >3mm - made threshold even higher so it happens less often
            1.5 * (1.0 + (elapsed_secs * 3.7).sin() * 0.3)
        } else if spike_trigger < -0.99 {
            // Spike down to ~1mm - made threshold even lower so it happens less often
            -0.8 * (1.0 + (elapsed_secs * 2.3).sin() * 0.2)
        } else {
            0.0
        };
        
        // Apply smoothing to spike magnitude to avoid sudden jumps
        let smoothed_spike = spike_magnitude * 0.05; // Gradual application, very smooth
        
        // Combine all components
        let diameter = base_diameter + slow_wave + medium_wave + fast_wave + smoothed_spike;
        
        // Clamp to reasonable bounds (0.5mm to 4.0mm)
        let diameter = diameter.max(0.5).min(4.0);
        
        // Add small random variation to x and y to create slight elliptical shape
        // but keep them close to the diameter
        let x_variation = 0.01 * ((elapsed_secs * 2.7).sin());
        let y_variation = 0.01 * ((elapsed_secs * 3.1).cos());
        
        let x_diameter = diameter + x_variation;
        let y_diameter = diameter + y_variation;
        
        self.data = Some(MockLaserData {
            diameter: Length::new::<millimeter>(diameter),
            x_axis: Some(Length::new::<millimeter>(x_diameter)),
            y_axis: Some(Length::new::<millimeter>(y_diameter)),
            last_timestamp: now,
        });
    }
}
