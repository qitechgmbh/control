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