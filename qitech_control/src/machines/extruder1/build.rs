use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use qitech_framework::machine::BuildContext;
use qitech_framework::machine::BuildError;
use qitech_framework::machine::BuildResult;
use qitech_framework::machine::MachineBuild;
use qitech_lib::ethercat_hal::EtherCATThreadChannel;
use qitech_lib::ethercat_hal::coe::ConfigurableDevice;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::ek1100::EK1100;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el2004::EL2004;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el3021::EL3021;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el3204::EL3204;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el6021::{EL6021, EL6021Configuration};
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::serial_interface::SerialInterfaceDevice;
use qitech_lib::ethercat_hal::io::temperature_input::TemperatureInputDevice;
use qitech_lib::units::{ThermodynamicTemperature, thermodynamic_temperature::degree_celsius};

use crate::machines::extruder1::mitsubishi_cs80::MitsubishiCS80;
use crate::machines::extruder1::screw_speed_controller::ScrewSpeedController;
use crate::machines::extruder1::temperature_controller::{
    TemperatureController, TemperatureControllerConfig,
};
use crate::machines::extruder1::{Extruder, Mode, VARIANT_V1, VARIANT_V2, Zone};
use crate::transmission::fixed::FixedTransmission;

/// EtherCAT roles, motor poles and gearing of one extruder generation.
struct Layout {
    ek1100_role: u16,
    serial_role: u16,
    digital_out_role: u16,
    pressure_sensor_role: u16,
    temperature_role: u16,
    motor_poles: usize,
    transmission: FixedTransmission,
}

const LAYOUT_V1: Layout = Layout {
    ek1100_role: 0,
    serial_role: 2,
    digital_out_role: 3,
    pressure_sensor_role: 4,
    temperature_role: 5,
    motor_poles: 4,
    transmission: FixedTransmission::new(1.0 / 34.0),
};

const LAYOUT_V2: Layout = Layout {
    ek1100_role: 0,
    serial_role: 1,
    digital_out_role: 2,
    pressure_sensor_role: 3,
    temperature_role: 4,
    motor_poles: 2,
    transmission: FixedTransmission::new(1.0 / 30.0),
};

impl MachineBuild for Extruder<VARIANT_V1> {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        Self::assemble(ctx, LAYOUT_V1)
    }
}

impl MachineBuild for Extruder<VARIANT_V2> {
    fn build(ctx: &mut BuildContext) -> BuildResult<Self> {
        Self::assemble(ctx, LAYOUT_V2)
    }
}

impl<const VARIANT: usize> Extruder<VARIANT> {
    fn assemble(ctx: &mut BuildContext, layout: Layout) -> BuildResult<Self> {
        let interface = ctx.get_ethercat_interface()?;

        // --- hardware ---
        ctx.find_ethercat_device_and_addr::<EK1100>(layout.ek1100_role)?;

        let temperature_input: Rc<RefCell<dyn TemperatureInputDevice>> =
            init_dc_sync::<EL3204>(ctx, &interface, layout.temperature_role)?;
        let pressure_sensor: Rc<RefCell<dyn AnalogInputDevice>> =
            init_dc_sync::<EL3021>(ctx, &interface, layout.pressure_sensor_role)?;
        let relais_output: Rc<RefCell<dyn DigitalOutputDevice>> =
            init_dc_sync::<EL2004>(ctx, &interface, layout.digital_out_role)?;
        let serial_interface: Rc<RefCell<dyn SerialInterfaceDevice>> =
            init_el6021(ctx, &interface, layout.serial_role)?;

        // --- commands ---
        ctx.command("mode.standby")
            .execute(|m: &mut Self| m.set_mode(Mode::Standby))
            .build()?;

        ctx.command("mode.heat")
            .execute(|m: &mut Self| m.set_mode(Mode::Heat))
            .build()?;

        ctx.command("mode.extrude")
            .execute(|m: &mut Self| m.set_mode(Mode::Extrude))
            .build()?;

        ctx.command("inverter.reset")
            .execute(Self::reset_inverter)
            .build()?;

        ctx.command("pressure.autotune.start")
            .can_execute(Self::can_autotune)
            .execute(Self::start_autotune)
            .build()?;

        ctx.command("pressure.autotune.stop")
            .execute(Self::stop_autotune)
            .build()?;

        // --- components ---
        let max_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);

        // Only front heating on: These values work 0.08, 0.001, 0.007, Overshoot 0.5
        // undershoot ~0.7 (Problems when starting far away because of integral)
        let zone_gains = (0.16, 0.0, 0.008);

        let temperature_controller_front =
            init_heating_zone(ctx, Zone::Front, max_temperature, zone_gains, 700.0, 1.0, 0)?;
        let temperature_controller_middle = init_heating_zone(
            ctx,
            Zone::Middle,
            max_temperature,
            zone_gains,
            700.0,
            1.0,
            1,
        )?;
        let temperature_controller_back =
            init_heating_zone(ctx, Zone::Back, max_temperature, zone_gains, 700.0, 1.0, 2)?;
        let temperature_controller_nozzle = init_heating_zone(
            ctx,
            Zone::Nozzle,
            max_temperature,
            zone_gains,
            200.0,
            0.95,
            3,
        )?;

        let screw_speed_controller = ScrewSpeedController::init::<VARIANT>(
            ctx,
            MitsubishiCS80::new(),
            layout.transmission,
            layout.motor_poles,
        )?;

        Ok(Self {
            relais_output,
            temperature_input,
            serial_interface,
            pressure_sensor,

            screw_speed_controller,
            temperature_controller_front,
            temperature_controller_middle,
            temperature_controller_back,
            temperature_controller_nozzle,

            nozzle_temperature_target_enabled: ctx
                .config::<bool>("heating.nozzle.target_enabled")
                .default(true)
                .build()?,

            mode: ctx.state::<Mode>("mode").build()?,

            combined_power: ctx.measurement::<f64>("power.combined").build()?,
            total_energy: ctx.measurement::<f64>("energy.total").build()?,

            last_energy_calculation_time: None,
        })
    }
}

// --- components ---

#[allow(clippy::too_many_arguments)]
fn init_heating_zone(
    ctx: &mut BuildContext,
    zone: Zone,
    max_temperature: ThermodynamicTemperature,
    gains: (f64, f64, f64),
    heating_element_wattage: f64,
    max_clamp: f64,
    port: usize,
) -> BuildResult<TemperatureController> {
    TemperatureController::init(
        ctx,
        zone,
        TemperatureControllerConfig {
            max_temperature,
            pwm_period: Duration::from_millis(500),
            heating_element_wattage,
            max_clamp,
            digital_port: port,
            temperature_port: port,
            gains,
        },
    )
}

// --- hardware ---

fn init_dc_sync<T>(
    ctx: &BuildContext,
    interface: &EtherCATThreadChannel,
    role: u16,
) -> BuildResult<Rc<RefCell<T>>>
where
    T: qitech_lib::ethercat_hal::devices::EthercatDevice,
{
    let (device, address) = ctx.find_ethercat_device_and_addr::<T>(role)?;

    interface
        .enable_dc_sync0(address)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    Ok(device)
}

fn init_el6021(
    ctx: &BuildContext,
    interface: &EtherCATThreadChannel,
    role: u16,
) -> BuildResult<Rc<RefCell<EL6021>>> {
    let (device, address) = ctx.find_ethercat_device_and_addr::<EL6021>(role)?;

    device
        .borrow_mut()
        .write_config(interface.clone(), address, &EL6021Configuration::default())
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    interface
        .enable_dc_sync0(address)
        .map_err(|e| BuildError::EtherCATConfigureError(e.to_string()))?;

    Ok(device)
}
