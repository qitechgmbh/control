use std::io;
use std::time::{Duration, Instant};

use modbus::WithContext;
use modbus::rtu::{FrameParseError, Transport as TransportTrait};
use modbus::rtu::{DispatchError, ReceiveError};
use modbus::rtu::Frame;

#[derive(Debug)]
pub struct CustomTransport
{
    port: std::sync::Mutex<Box<dyn serialport::SerialPort>>,
    slave_id: u8,
}

impl CustomTransport
{
    pub fn new(port: Box<dyn serialport::SerialPort>, slave_id: u8) -> Self
    {
        let p = std::sync::Mutex::new(port);

        Self { port: p ,slave_id }
    }
}

// implement InterfaceTransport for your RTU frames
impl TransportTrait<()> for CustomTransport 
{
    type Error = io::Error;

    fn slave_id(&self) -> u8 { self.slave_id }

    fn try_send(
        &mut self,
        item: WithContext<Frame, ()>
    ) -> Result<(), DispatchError<Self::Error>> 
    {
        let mut port = self.port.lock().unwrap();

        match port.write_all(item.payload.bytes()) 
        {
            Ok(_) => {},
            Err(e) => 
            {
                tracing::error!("Error: try_send: {:?}", e);

                return Err(DispatchError::BridgeDropped);
            },
        }

        match port.flush() 
        {
            Ok(_) => {},
            Err(e) => 
            {
                tracing::error!("Error: try_send: {:?}", e);

                return Err(DispatchError::BridgeDropped);
            },
        }

        Ok(())
    }

    fn try_recv(
        &mut self
    ) -> Result<Option<Frame>, ReceiveError<Self::Error>> 
    {
        let mut port = self.port.lock().unwrap();

        let mut buf = [0u8; 256];
        match port.read(&mut buf) 
        {
            Ok(0) => Ok(None),
            Ok(n) => 
            {
                match Frame::from_bytes(&buf[..n]) 
                {
                    Ok(frame) => Ok(Some(frame)),
                    Err(e) => 
                    {
                        Err(ReceiveError::ParseError(e))
                    },
                }
            } 
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(ReceiveError::Transport(e)),
        }
    }

    fn send(
        &mut self,
        item: WithContext<Frame, ()>
    ) -> Result<(), DispatchError<Self::Error>> 
    {
        let mut port = self.port.lock().unwrap();

        match port.write_all(item.payload.bytes()) 
        {
            Ok(_) => {},
            Err(e) => 
            {
                tracing::error!("Error: try_send: {:?}", e);
            },
        }

        match port.flush() 
        {
            Ok(_) => {},
            Err(e) => 
            {
                tracing::error!("Error: try_send: {:?}", e);
            },
        }

        Ok(())
    }

    fn recv(&mut self) -> Result<Frame, ReceiveError<Self::Error>> 
    {
        let mut port = self.port.lock().unwrap();

        let mut buf = [0u8; 256];
        let mut pos = 0;

        let mut now: Instant = Instant::now();
        let timeout = Duration::from_secs(1);

        loop
        {
            match port.read(&mut buf[pos..])
            {
                Ok(0) => 
                {
                    return Err(ReceiveError::ParseError(FrameParseError::TooSmall(0)));
                }
                Ok(n) => 
                {
                    pos += n;
                    
                    if let Ok(frame) = Frame::from_bytes(&buf[..pos])
                    {
                        return Ok(frame);
                    }
                    
                    now = Instant::now(); // reset timeout timer
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => 
                {
                    if now.elapsed() >= timeout 
                    { 
                        if pos > 0 
                        {
                            match Frame::from_bytes(&buf[..pos])
                            {
                                Ok(frame) => { return Ok(frame); },
                                Err(e) => { return Err(ReceiveError::ParseError(e)); },
                            }
                        }
                        
                        return Err(ReceiveError::Timeout); 
                    }
                    
                    std::thread::sleep(Duration::from_millis(150));
                }
                Err(e) => return Err(ReceiveError::Transport(e)),
            }
        }
    }
}
