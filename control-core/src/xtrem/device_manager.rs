use std::{any::TypeId, io, net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket}, time::Duration};

use crossbeam::channel::{Sender, Receiver};

use crate::xtrem::protocol::{frame, request};

pub struct DeviceManager 
{
    // udp socket for making requests
    socket: UdpSocket,

    // port for sending request
    port_tx: u16,

    // list of all identified and managed devices
    devices: heapless::Vec<DeviceEntry, { Self::SUPPORTED_DEVICES_COUNT_MAX }>
}

pub struct DeviceEntry 
{
    // uid of the device, or in this case the serial number
    uid: u64,

    /// Type id of the underlying type
    type_id: TypeId,

    /// uid of the machine this device is assigned to  
    machine_uid: u64, 

    // underlying device
    device: ConcreteDevice,

    /// Queue used by manager to receive requests
    rx_queue: Receiver<Request>,

    /// Queue used by manager to send responses
    tx_queue: Sender<Response>,
}

pub enum ConcreteDevice {
    // Zebra(XtremZebra)
}

impl DeviceManager {
    const DEVICE_ID: u8 = 0x00;
    const SUPPORTED_DEVICES_COUNT_MAX: usize = 16;

    pub fn new(ip_addr: u32, port_rx: u16, port_tx: u16) -> io::Result<Self> {
        let ip = Ipv4Addr::from(ip_addr);
        let addr = SocketAddr::new(IpAddr::V4(ip), port_rx);

        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            port_tx,
            devices: Default::default(),
        })
    }

    pub fn update(&mut self) {

    }

    pub fn detect_devices(&mut self) -> io::Result<()> {

        // device id to broadcast to all devices in the ip range
        const DEVICE_ID_BROADCAST: u8 = 255;

        const TIMEOUT_DURATION: Duration = Duration::from_millis(2000);

        let request = request::serial_number::read();

        let mut buf = [0_u8; 4096];
        let data = frame::encode(Self::DEVICE_ID, DEVICE_ID_BROADCAST, request, &mut buf);

        // send broadcast message to all devices on subnet to read serial_number (uid)
        self.socket.send(data)?;

        std::thread::sleep(TIMEOUT_DURATION);

        loop {
            let len = self.socket.recv(&mut buf)?;

            if len == 0 {
                break;
            }


        }

        Ok(())
    }
}