use super::devices::el2008::EL2008;
use crate::ethercat::config::{MAX_SUBDEVICES, PDI_LEN};
use anyhow::anyhow;
use ethercrab::{subdevice_group::Op, MainDevice, SubDeviceGroup, SubDevicePdi, SubDeviceRef};
use std::{any::Any, sync::Arc};
use tokio::sync::RwLock;

pub trait Device: Any + Send + Sync {
    /// Input data from the last cycle
    /// `ts` is the timestamp when the input data was sent by the device
    fn input(&mut self, _input: &[u8]) {
        ()
    }

    /// The accepted length of the input data
    fn input_len(&self) -> usize {
        0
    }

    /// automatically validate input length, then calls input
    fn input_checked(&mut self, input: &[u8]) -> Result<(), anyhow::Error> {
        // validate input has correct length
        let input_len = self.input_len();
        if input.len() != input_len {
            return Err(anyhow::anyhow!(
                "Input length is {} and must be {} bytes",
                input.len(),
                input_len
            ));
        }

        self.input(input);

        Ok(())
    }

    /// Output data for the next cycle
    /// `ts` is the timestamp when the output data is predicted to be received by the device
    fn output(&self, _output: &mut [u8]) {
        ()
    }

    /// The accepted length of the output data
    fn output_len(&self) -> usize {
        0
    }

    fn output_checked(&self, output: &mut [u8]) -> Result<(), anyhow::Error> {
        // validate input has correct length
        let output_len = self.output_len();
        if output.len() != output_len {
            return Err(anyhow::anyhow!(
                "Output length is {} and must be {} bytes",
                output.len(),
                output_len
            ));
        }

        self.output(output);

        Ok(())
    }

    /// Write timestamps for current cycle
    fn ts(&mut self, _input_ts: u64, _output_ts: u64) {
        ()
    }

    fn as_any(&self) -> &dyn Any;
}

pub async fn downcast_device<T: Device>(
    device: Arc<RwLock<dyn Device>>,
) -> Result<Arc<RwLock<T>>, anyhow::Error> {
    // Acquire a read lock on the RwLock
    let read_lock = device.read().await;

    // Check if the inner type can be downcasted to T
    if read_lock.as_any().is::<T>() {
        // Clone the Arc and return it as the desired type
        let cloned_device = Arc::clone(&device);
        // Transmute the Arc to the desired type
        unsafe {
            Ok(Arc::from_raw(
                Arc::into_raw(cloned_device) as *const RwLock<T>
            ))
        }
    } else {
        Err(anyhow!("Couln't downcast device to desired type"))
    }
}

fn device_from_name(value: &str) -> Result<Arc<RwLock<dyn Device>>, anyhow::Error> {
    match value {
        "EL2008" => Ok(Arc::new(RwLock::new(EL2008::new()))),
        _ => Err(anyhow::anyhow!("No Driver: {}", value)),
    }
}

fn device_from_subdevice<'maindevice, 'group>(
    subdevice: &SubDeviceRef<'maindevice, SubDevicePdi<PDI_LEN>>,
) -> Result<Arc<RwLock<dyn Device>>, anyhow::Error> {
    device_from_name(subdevice.name())
}

pub fn devices_from_subdevice_group<'maindevice, 'group>(
    group: &SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op>,
    maindevice: &MainDevice,
) -> Vec<Option<Arc<RwLock<dyn Device>>>> {
    group
        .iter(maindevice)
        .map(|subdevice| device_from_subdevice(&subdevice).ok())
        .collect()
}

pub async fn get_device<DEVICE: Device>(
    devices: &Vec<Option<Arc<RwLock<dyn Device>>>>,
    index: usize,
) -> Result<Arc<RwLock<DEVICE>>, anyhow::Error> {
    let x = downcast_device::<DEVICE>(devices[index].as_ref().unwrap().clone()).await;
    x
}
