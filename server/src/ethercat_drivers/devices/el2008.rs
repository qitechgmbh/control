use std::{future::Future, pin::Pin, sync::Arc};

use crate::ethercat_drivers::io::digital_output::{DigitalOutput, DigitalOutputGet};

use super::Device;

#[derive(Debug, Clone, Copy)]
pub enum EL2008Port {
    Pin1,
    Pin2,
    Pin3,
    Pin4,
    Pin5,
    Pin6,
    Pin7,
    Pin8,
}

impl EL2008Port {
    pub fn to_bit_index(&self) -> usize {
        match self {
            EL2008Port::Pin1 => 0,
            EL2008Port::Pin2 => 1,
            EL2008Port::Pin3 => 2,
            EL2008Port::Pin4 => 3,
            EL2008Port::Pin5 => 4,
            EL2008Port::Pin6 => 5,
            EL2008Port::Pin7 => 6,
            EL2008Port::Pin8 => 7,
        }
    }
}

const OUTPUT_PDU_LEN: usize = 1;
const INPUT_PDU_LEN: usize = 0;

#[derive(Debug)]
pub struct EL2008 {
    output_pdus: [u8; OUTPUT_PDU_LEN],
    pub output_ts: u64,
}

impl EL2008 {
    pub fn new() -> Self {
        Self {
            output_pdus: [0; OUTPUT_PDU_LEN],
            output_ts: 0,
        }
    }

    fn write(&mut self, port: EL2008Port, signal: bool) {
        let pdu = match signal {
            true => 0x01,
            false => 0x00,
        };
        let bit_index = port.to_bit_index();
        self.output_pdus[0] = (self.output_pdus[0] & !(1 << bit_index)) | (pdu << bit_index);
    }

    fn get(&self, port: EL2008Port) -> DigitalOutputGet {
        let bit_index = port.to_bit_index();
        DigitalOutputGet {
            ts: self.output_ts,
            value: self.output_pdus[0] & (1 << bit_index) != 0,
        }
    }

    /// Create closures to interface a single digital output pin
    pub fn digital_output(
        device: Arc<tokio::sync::RwLock<EL2008>>,
        port: EL2008Port,
    ) -> DigitalOutput {
        // build async write closure
        let device1 = device.clone();
        let write = Box::new(move |signal| {
            let device1 = device1.clone();
            Box::pin(async move {
                let mut device1_guard = device1.write().await;
                device1_guard.write(port, signal)
            }) as Pin<Box<dyn Future<Output = ()> + Send + 'static>>
        });

        // build async get closure
        let device2 = device.clone();
        let get = Box::new(move || {
            let device2 = device2.clone();
            Box::pin(async move {
                let device2_guard = device2.read().await;
                device2_guard.get(port)
            }) as Pin<Box<dyn Future<Output = DigitalOutputGet> + Send + 'static>>
        });

        DigitalOutput { write, get }
    }
}

impl Device for EL2008 {
    fn input(&mut self, _ts: u64, input: &[u8]) -> Result<(), anyhow::Error> {
        // validate input has correct length
        if input.len() != INPUT_PDU_LEN {
            return Err(anyhow::anyhow!(
                "Input length is {} and must be {} bytes",
                input.len(),
                INPUT_PDU_LEN
            ));
        }

        Ok(())
    }
    fn output(&self, _ts: u64, output: &mut [u8]) -> Result<(), anyhow::Error> {
        // validate input has correct length
        if output.len() != OUTPUT_PDU_LEN {
            return Err(anyhow::anyhow!(
                "Output length is {} and must be {} bytes",
                output.len(),
                OUTPUT_PDU_LEN
            ));
        }

        output.copy_from_slice(&self.output_pdus);

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
