use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use smol::Timer;
use smol::future::FutureExt;
use smol::io;
use smol::io::AsyncReadExt;
use smol::io::AsyncWriteExt;
use smol::net::TcpStream;

const PROTOCOL_ID: u16 = 0;
const UNIT_ID: u8 = 0;
const READ_HOLDING_FUNCTION_CODE: u8 = 3;
const WRITE_HOLDING_FUNCTION_CODE: u8 = 16;
const READ_HOLDING_LENGTH: u16 = 6; // Unit ID 00 + Func Code 03 + Start Address XXXX + Quantity XXXX = 6 bytes
const WRITE_HOLDING_LENGTH_WITHOUT_DATA: u16 = 6; // Unit ID 00 + Func Code 16 + Start Address XXXX + Quantity XXXX = 6 bytes

struct Packet {
    buf: Vec<u8>,
}

impl Packet {
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn add_u8(&mut self, x: u8) {
        self.buf.push(x);
    }

    pub fn add_u16(&mut self, x: u16) {
        let buf = x.to_be_bytes();
        self.buf.push(buf[0]);
        self.buf.push(buf[1]);
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.buf
    }
}

pub struct ModbusTcpDevice {
    stream: TcpStream,
    transactions: u16,
}

impl Debug for ModbusTcpDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.stream.peer_addr() {
            Ok(addr) => write!(f, "ModbusTcpDevice({:?})", addr),
            Err(_) => std::fmt::Result::Err(std::fmt::Error),
        }
    }
}

impl ModbusTcpDevice {
    pub async fn new(addr: SocketAddr) -> Result<Self> {
        let timeout = async {
            Timer::after(Duration::from_millis(10)).await;
            Err(io::Error::from(ErrorKind::TimedOut))
        };

        let stream = TcpStream::connect(addr)
            .or(timeout)
            .await
            .context("Could not connect to modbus device!")?;

        Ok(Self {
            stream,
            transactions: 0,
        })
    }

    async fn read_and_check_responce_header(&mut self, func_code: u8) -> Result<()> {
        assert_eq!(
            self.read_u16().await?,
            self.transactions,
            "Modbus device sent unexpected transaction id!"
        );

        assert_eq!(
            self.read_u16().await?,
            PROTOCOL_ID,
            "Modbus device sent unexpected protocol id!"
        );

        let length = self.read_u16().await?;
        assert!(
            length >= 2,
            "Modbus device will send too few bytes to understand the response!"
        );

        assert_eq!(
            self.read_u8().await?,
            UNIT_ID,
            "Modbus device sent unexpected unit id!"
        );

        let func_code_res = self.read_u8().await?;
        if func_code != func_code_res {
            let error_code = self.read_u8().await?;
            bail!("Modbus device sent unexpected function code 0x{:x}! It probably encountered an error. Error code: 0x{:x}", func_code_res, error_code);
        }

        Ok(())
    }

    async fn send_read_holding_request(&mut self, addr: u16, count: u16) -> Result<u16> {
        let mut packet = Packet::new();

        self.transactions += 1;
        packet.add_u16(self.transactions);
        packet.add_u16(PROTOCOL_ID);
        packet.add_u16(READ_HOLDING_LENGTH);

        packet.add_u8(UNIT_ID);
        packet.add_u8(READ_HOLDING_FUNCTION_CODE);

        packet.add_u16(addr);
        packet.add_u16(count);

        self.send(packet).await?;
        self.read_and_check_responce_header(READ_HOLDING_FUNCTION_CODE).await?;

        let num_data_bytes = self.read_u8().await? as u16;
        assert_eq!(
            count * 2,
            num_data_bytes,
            "Modbus device wants to send only a portion of the requested range - UNIMPLEMENTED!"
        );

        Ok(num_data_bytes)
    }

    pub async fn get_holding_registers(&mut self, addr: u16, count: u16) -> Result<Vec<u16>> {
        self.send_read_holding_request(addr, count).await?;

        let mut res: Vec<u16> = Vec::new();
        while res.len() < count as usize {
            res.push(self.read_u16().await?);
        }

        Ok(res)
    }

    pub async fn set_holding_registers(&mut self, addr: u16, values: &[u16]) -> Result<()> {
        let mut packet = Packet::new();
        let count: u16 = values.len() as u16;

        self.transactions += 1;
        packet.add_u16(self.transactions);
        packet.add_u16(PROTOCOL_ID);
        packet.add_u16(WRITE_HOLDING_LENGTH_WITHOUT_DATA + 2 * count);

        packet.add_u8(UNIT_ID);
        packet.add_u8(WRITE_HOLDING_FUNCTION_CODE);

        packet.add_u16(addr);
        packet.add_u16(2 * count);

        for value in values {
            packet.add_u16(value.to_owned());
        }

        self.send(packet).await?;
        self.read_and_check_responce_header(WRITE_HOLDING_FUNCTION_CODE).await?;

        assert_eq!(
            self.read_u16().await?,
            addr,
            "Modbus device wrote to wrong register address!"
        );

        let _count_written = self.read_u16().await?;

        Ok(())
    }

    pub async fn get_string<const N: usize>(&mut self, addr: u16) -> Result<String> {
        assert!(N.is_multiple_of(2), "Strings are always of even length!");
        assert!(N <= 256, "Cannot read long strings in a single swoop!");
        let count = N / 2;

        self.send_read_holding_request(addr, count as u16).await?;
        let buf = self.read_bytes::<N>().await?;

        let s = String::from_utf8_lossy(&buf);
        let s = match s.split_once('\0') {
            None => s.to_string(),
            Some((l, _)) => l.to_string(),
        };

        Ok(s)
    }

    pub async fn set_string<const N: usize>(&mut self, _addr: u16, s: &str) -> Result<()> {
        assert!(s.len() <= N, "String is too long to fit!");

        Ok(())
    }

    pub async fn get_u32(&mut self, addr: u16) -> Result<u32> {
        let length = self.send_read_holding_request(addr, 2).await?;
        assert_eq!(
            length, 4,
            "Expected modbus device to send exactly 4 bytes of data!"
        );

        let buf = self.read_bytes::<4>().await?;
        Ok(u32::from_be_bytes(buf))
    }

    // We have to send the request in a single packet,
    // otherwise the device may fail to respond correctly.
    async fn send(&mut self, packet: Packet) -> Result<()> {
        self.stream
            .write_all(packet.as_bytes())
            .await
            .context("Could write to modbus device!")
    }

    async fn read_u8(&mut self) -> Result<u8> {
        let buf = self.read_bytes::<1>().await?;
        Ok(u8::from_be_bytes(buf))
    }

    async fn read_u16(&mut self) -> Result<u16> {
        let buf = self.read_bytes::<2>().await?;
        Ok(u16::from_be_bytes(buf))
    }

    async fn read_bytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        let mut buf = [0; N];
        self.stream
            .read_exact(&mut buf)
            .await
            .context("Could not read from modbus device!")?;
        Ok(buf)
    }
}
