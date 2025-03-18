use std::error::Error;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
use tokio_modbus::prelude::*;
use tokio_serial::{SerialPortInfo, SerialPortType, SerialStream};

#[derive(Clone)]
struct DreConfig {
    lower_tolerance: f32,
    target_diameter: f32,
    upper_tolerance: f32,
}

struct DreStatus {
    hist_timestamps: Vec<u64>,
    hist_diameter: Vec<f32>,
}

struct Dre {
    ctx: client::Context,
    diameter: f32,
    status: DreStatus,
    config: DreConfig,
    path: String,
    failed_request_counter: u8,
}

impl Dre {
    async fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        println!("Attempting to connect to DRE at {}", path);
        
        // Create a serial port with tokio support
        let builder = tokio_serial::new(path, 38400)
            .data_bits(tokio_serial::DataBits::Eight)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_secs(1));
        
        let port = SerialStream::open(&builder)?;
        
        // Create modbus context
        let mut ctx = client::rtu::attach_slave(port, Slave(1));
        
        // Read initial configuration
        // Read upper tolerance, target diameter, and lower tolerance
        let upper_tolerance = ctx.read_holding_registers(102, 1).await?[0] as f32 / 1000.0;
        let target_diameter = ctx.read_holding_registers(101, 1).await?[0] as f32 / 1000.0;
        let lower_tolerance = ctx.read_holding_registers(103, 1).await?[0] as f32 / 1000.0;
        
        println!("Connected to DRE device with configuration:");
        println!("  Target diameter: {:.3} mm", target_diameter);
        println!("  Upper tolerance: {:.3} mm", upper_tolerance);
        println!("  Lower tolerance: {:.3} mm", lower_tolerance);
        
        Ok(Self {
            ctx,
            diameter: 0.0,
            status: DreStatus {
                hist_timestamps: Vec::new(),
                hist_diameter: Vec::new(),
            },
            config: DreConfig {
                lower_tolerance,
                target_diameter,
                upper_tolerance,
            },
            path: path.to_string(),
            failed_request_counter: 0,
        })
    }
    
    async fn request_diameter(&mut self) -> Result<f32, Box<dyn Error>> {
        match self.ctx.read_input_registers(0, 1).await {
            Ok(registers) => {
                let raw_value = registers[0];
                let diameter = raw_value as f32 / 1000.0;
                self.diameter = diameter;
                
                // Record in history
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis() as u64;
                
                self.status.hist_timestamps.push(now);
                self.status.hist_diameter.push(diameter);
                
                // Reset error counter
                self.failed_request_counter = 0;
                
                println!("Current diameter: {:.3} mm", diameter);
                Ok(diameter)
            },
            Err(e) => {
                self.failed_request_counter += 1;
                if self.failed_request_counter >= 5 {
                    return Err(format!("Too many failed requests ({}). Disconnecting.", 
                        self.failed_request_counter).into());
                }
                Err(format!("Failed to read diameter: {}", e).into())
            }
        }
    }
    
    async fn reload_config(&mut self) -> Result<DreConfig, Box<dyn Error>> {
        // Read upper tolerance, target diameter, and lower tolerance
        let upper_tolerance = self.ctx.read_holding_registers(102, 1).await?[0] as f32 / 1000.0;
        let target_diameter = self.ctx.read_holding_registers(101, 1).await?[0] as f32 / 1000.0;
        let lower_tolerance = self.ctx.read_holding_registers(103, 1).await?[0] as f32 / 1000.0;
        
        self.config = DreConfig {
            lower_tolerance,
            target_diameter,
            upper_tolerance,
        };
        
        println!("Reloaded configuration:");
        println!("  Target diameter: {:.3} mm", target_diameter);
        println!("  Upper tolerance: {:.3} mm", upper_tolerance);
        println!("  Lower tolerance: {:.3} mm", lower_tolerance);
        
        Ok(self.config.clone())
    }
    
    fn get_average_diameter(&self) -> f32 {
        let len = self.status.hist_diameter.len();
        if len == 0 {
            return 0.0;
        }
        
        let last_n = if len > 20 { 20 } else { len };
        let sum: f32 = self.status.hist_diameter[len - last_n..].iter().sum();
        let avg = sum / last_n as f32;
        
        println!("Average diameter (last {} readings): {:.3} mm", last_n, avg);
        avg
    }
    
    fn is_valid_port(port_info: &SerialPortInfo) -> bool {
        match &port_info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                // Check serial number if available
                if let Some(serial) = &usb_info.serial_number {
                    if serial.to_lowercase().starts_with("dre") {
                        return true;
                    }
                    if serial == "A50285BI" {
                        return true;
                    }
                }
                
                // Check vendor and product IDs
                if usb_info.vid == 0x0403 && usb_info.pid == 0x6001 {
                    return true;
                }
                
                false
            },
            _ => false,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // List available ports
    println!("Available serial ports:");
    let ports = tokio_serial::available_ports()?;
    
    if ports.is_empty() {
        println!("No serial ports found!");
        return Ok(());
    }
    
    let mut valid_ports = Vec::new();
    
    for port in &ports {
        println!("  {}", port.port_name);
        if Dre::is_valid_port(port) {
            valid_ports.push(port.port_name.clone());
            println!("    ✓ Potentially a DRE device");
        }
    }
    
    // Try to connect to a valid port
    let mut dre = None;
    
    for port in valid_ports {
        match Dre::new(&port).await {
            Ok(device) => {
                dre = Some(device);
                println!("Successfully connected to DRE on {}", port);
                break;
            },
            Err(e) => {
                println!("Failed to connect to {} as DRE: {}", port, e);
            }
        }
    }
    
    let mut dre = match dre {
        Some(d) => d,
        None => {
            println!("No DRE devices found. Exiting.");
            return Ok(());
        }
    };
    
    // Main monitoring loop - read diameter every 500ms
    for _ in 0..20 {
        match dre.request_diameter().await {
            Ok(_) => {
                // After every 5 readings, calculate average
                if dre.status.hist_diameter.len() % 5 == 0 {
                    dre.get_average_diameter();
                }
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
        
        // Wait 500ms before next reading
        time::sleep(Duration::from_millis(500)).await;
    }
    
    // Try reloading configuration
    println!("\nReloading configuration...");
    match dre.reload_config().await {
        Ok(_) => println!("Configuration reloaded successfully."),
        Err(e) => println!("Failed to reload configuration: {}", e),
    }
    
    println!("Prototype run completed. Closing connection.");
    Ok(())
}
