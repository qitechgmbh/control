use std::{cell::RefCell, rc::Rc, time::{Duration, Instant}};
use control_core::transmission::fixed::FixedTransmission;
use qitech_lib::{ethercat_hal::{coe::ConfigurableDevice, devices::{ek1100::EK1100, el2004::EL2004, el3021::EL3021, el3204::EL3204, el6021::{EL6021, EL6021Configuration}}, io::{analog_input::AnalogInputDevice, digital_output::DigitalOutputDevice, serial_interface::SerialInterfaceDevice, temperature_input::TemperatureInputDevice}}, machines::MachineIdentificationUnique, units::{AngularVelocity, Pressure, ThermodynamicTemperature, angular_velocity::revolution_per_minute, pressure::bar, thermodynamic_temperature::degree_celsius}};
use crate::{MACHINE_EXTRUDER_V1, MACHINE_EXTRUDER_V2, MachineHardware, MachineMessage, MachineNew};
use super::{ExtruderV2, Heating, api::ExtruderV2Namespace, mitsubishi_cs80::MitsubishiCS80, screw_speed_controller::ScrewSpeedController, temperature_controller::TemperatureController};

struct ExtruderRoles {
    temp_role : u16,
    ek1100_role : u16,
    pressure_sensor_role:u16,
    digital_out_role:u16,
    serial_role:u16,
}

impl ExtruderRoles {
    fn get_v3_roles() -> ExtruderRoles {
        ExtruderRoles {
            temp_role: 4,
            ek1100_role: 0,
            pressure_sensor_role: 3,
            digital_out_role:2,
            serial_role: 1,
        }
    }

    fn get_v2_roles() -> ExtruderRoles {
        ExtruderRoles {
            temp_role: 5,
            ek1100_role: 0,
            pressure_sensor_role: 4,
            digital_out_role:3,
            serial_role: 2,
        }
    }
}

impl MachineNew for ExtruderV2 {
    fn new(hw: MachineHardware) -> Result<Self, anyhow::Error> {
        let motor_poles;
        let transmission;
        
        let roles = match hw.identification.machine_ident.machine {
            MACHINE_EXTRUDER_V1 => {
                motor_poles = 4;
                transmission = FixedTransmission::new(1.0 / 34.0);
                ExtruderRoles::get_v2_roles()
            },
            MACHINE_EXTRUDER_V2 => {
                println!("Setting up like its V3");
                motor_poles = 2;
                transmission = FixedTransmission::new(1.0 / 30.0);
                ExtruderRoles::get_v3_roles()
            },
            _ => return Err(anyhow::anyhow!("ExtruderV2 Unexpected Machine: {}",hw.identification.machine_ident.machine)),
        };

        let interface = match hw.ethercat_interface.clone() {
            Some(interface) => interface,
            None => return Err(anyhow::anyhow!("No Ethercat Interface was supplied, but is required to setup Extruder")),
        };


        let _ek1100 : Rc<RefCell<EK1100>> = hw.try_get_ethercat_device_by_role(roles.ek1100_role)?;    
        let temperature_device : Rc<RefCell<dyn TemperatureInputDevice>> = hw.try_get_ethercat_device_by_role::<EL3204>(roles.temp_role)?;        
        let pressure_sensor : Rc<RefCell<dyn AnalogInputDevice>> =  hw.try_get_ethercat_device_by_role::<EL3021>(roles.pressure_sensor_role)?;
        let digital_out_device : Rc<RefCell<dyn DigitalOutputDevice>> = hw.try_get_ethercat_device_by_role::<EL2004>(roles.digital_out_role)?;
        let serial_device : Rc<RefCell<EL6021>> = hw.try_get_ethercat_device_by_role::<EL6021>(roles.serial_role)?;
        let el6021_addr = hw.try_get_ethercat_meta_by_role(roles.serial_role)?;
        let mut el6021 = serial_device.borrow_mut();        
        let _res = el6021.write_config(interface.clone(),el6021_addr,&EL6021Configuration::default());
        drop(el6021);

        let extruder_max_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);
        let temperature_controller_front = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature, 
            Heating::default(),
            Duration::from_millis(500),
            700.0,
            1.0,
            0,
            0
        );

        let temperature_controller_middle = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            Heating::default(),
            Duration::from_millis(500),
            700.0,
            1.0,
            1,
            1
        );

        let temperature_controller_back = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            Heating::default(),
            Duration::from_millis(500),
            700.0,
            1.0,
            2,
            2,
        );

        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5 undershoot ~0.7 (Problems when starting far away because of integral)
        let temperature_controller_nozzle = TemperatureController::new(
            0.16,
            0.0,
            0.008,
            ThermodynamicTemperature::new::<degree_celsius>(150.0),
            extruder_max_temperature,
            Heating::default(),
            Duration::from_millis(500),
            200.0,
            0.95,
            3,
            3,
        );

        let inverter = MitsubishiCS80::new();
        let target_pressure = Pressure::new::<bar>(0.0);
        let target_rpm = AngularVelocity::new::<revolution_per_minute>(0.0);
        
        let screw_speed_controller = ScrewSpeedController::new(
            inverter,
            target_pressure,
            target_rpm,
            transmission,
            motor_poles,
        );            
        let (tx, rx) = tokio::sync::mpsc::channel::<MachineMessage>(2);

        let mut extruder: ExtruderV2 = Self {
            api_receiver: rx,
            api_sender: tx,
            machine_identification_unique: hw.identification,
            namespace: ExtruderV2Namespace {
                namespace: None,
            },
            last_measurement_emit: Instant::now(),
            mode:  crate::extruder1::ExtruderV2Mode::Standby,
            total_energy_kwh: 0.0,
            last_energy_calculation_time: None,
            temperature_controller_front,
            temperature_controller_middle,
            temperature_controller_back,
            temperature_controller_nozzle,
            screw_speed_controller,
            emitted_default_state: false,
            last_status_hash: None,

            relais_output: digital_out_device,
            temperature_input: temperature_device,
            serial_interface: serial_device,
            pressure_sensor,
        };
        extruder.emit_state();
        Ok(extruder)        
    }
}
