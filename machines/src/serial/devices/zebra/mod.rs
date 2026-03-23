use std::sync::{Arc, atomic::AtomicBool};

mod types;

mod atomic_f64;
use anyhow::anyhow;
use atomic_f64::AtomicF64;
use control_core::xtrem_protocol::xtrem_serial_interface::{Frame, Function};
use smol::net::UdpSocket;

#[derive(Debug)]
pub struct XtremZebra {

    path: String,

    weight: Arc<AtomicF64>,

    // used to indicate other thread to terminate
    shutdown_flag: Arc<AtomicBool>,
}

impl XtremZebra {
    pub fn new(path: String, id: u32) -> aynhow::Result<Self> {


        Ok(())
    }
}

// helpers
impl XtremZebra {

    fn receive_data(&mut self, data: &[u8]) {

        let Some((id, weight))

    }

    async fn create_sockets(
        rx_port: u16,
        tx_addr: &str,
    ) -> anyhow::Result<(UdpSocket, UdpSocket)> {
        
        let sock_rx = UdpSocket::bind(("0.0.0.0", rx_port)).await?;
        let sock_tx = UdpSocket::bind("0.0.0.0").await?;
        sock_tx.set_broadcast(true);
        sock_tx.connect(tx_addr);        
        Ok((sock_rx, sock_tx))
    }

    pub fn decode_frame(data: &[u8]) -> anyhow::Result<Frame> {
        let ascii = String::from_utf8_lossy(&data);

        if ascii.len() < 10 {
            return Err(anyhow!("Too short ASCII frame"));
        }

        let id_origin = ascii[0..2].parse::<u8>().unwrap_or(0);
        let id_dest = ascii[2..4].parse::<u8>().unwrap_or(0);
        let func_char = ascii.chars().nth(4).unwrap_or('r');
        let function = Function::from_char(func_char)
            .ok_or_else(|| anyhow!("Invalid function char '{}'", func_char))?;

        let data_address = u16::from_str_radix(&ascii[5..9], 16).unwrap_or(0);
        let data_length = 0;
        let payload = Vec::new();

        let frame = Frame {
            stx: 0x02,
            id_origin,
            id_dest,
            function,
            data_address,
            data_length,
            data: payload,
            lrc: 0,
            etx: 0x03,
        };

        Ok(frame)
    }
}