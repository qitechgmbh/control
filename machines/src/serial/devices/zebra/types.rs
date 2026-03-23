use anyhow::anyhow;
use control_core::xtrem_protocol::xtrem_serial_interface::{DataAddress, Frame, Function};

#[derive(Debug, Clone)]
struct Request {
    pub id_origin: u8,
    pub id_dest: u8,
    pub data_address: DataAddress,
    pub function: Function,
    pub data: Vec<u8>,
}

impl Request {
    pub fn new() {

    }

    pub fn to_frame(self) -> Frame {
        let id_origin = self.id_origin;
        let id_dest = self.id_dest;
        let data_address = self.data_address.as_hex();
        let data_length = self.data.len() as u8;

        // Build frame body (everything between STX and ETX)
        let mut frame_body = Vec::new();
        frame_body.push(id_origin);
        frame_body.push(id_dest);
        frame_body.push(self.function.as_char() as u8);
        frame_body.extend_from_slice(&data_address.to_be_bytes());
        frame_body.push(data_length);
        frame_body.extend_from_slice(&self.data);

        let lrc = Frame::compute_lrc(&frame_body);

        Frame {
            stx: 0x02,
            id_origin,
            id_dest,
            function: self.function,
            data_address,
            data_length,
            data: self.data,
            lrc,
            etx: 0x03,
        }
    }
}