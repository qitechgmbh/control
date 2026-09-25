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
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el6021::EL6021;
use qitech_lib::ethercat_hal::devices::beckhoff_modules::el6021::EL6021Configuration;
use qitech_lib::ethercat_hal::io::analog_input::AnalogInputDevice;
use qitech_lib::ethercat_hal::io::digital_output::DigitalOutputDevice;
use qitech_lib::ethercat_hal::io::serial_interface::SerialInterfaceDevice;
use qitech_lib::ethercat_hal::io::temperature_input::TemperatureInputDevice;
use qitech_lib::units::ThermodynamicTemperature;
use qitech_lib::units::energy::kilowatt_hour;
use qitech_lib::units::power::watt;
use qitech_lib::units::thermodynamic_temperature::degree_celsius;

use crate::machines::extruder1::DEFAULT_MIN_EXTRUSION_TEMPERATURE;
use crate::machines::extruder1::Extruder;
use crate::machines::extruder1::HeatingAlgorithm;
use crate::machines::extruder1::Mode;
use crate::machines::extruder1::VARIANT_V1;
use crate::machines::extruder1::VARIANT_V2;
use crate::machines::extruder1::Zone;
use crate::machines::extruder1::heating_params::build_strategy;
use crate::machines::extruder1::mitsubishi_cs80::MitsubishiCS80;
use crate::machines::extruder1::screw_speed_controller::ScrewSpeedController;
use crate::machines::extruder1::temperature_controller::TemperatureController;
use crate::machines::extruder1::temperature_controller::TemperatureControllerConfig;
use crate::transmission::fixed::FixedTransmission;

/// EtherCAT roles, motor poles, gearing, heater bands and default heating
/// algorithm of one extruder generation.
struct Layout {
    ek1100_role: u16,
    serial_role: u16,
    digital_out_role: u16,
    pressure_sensor_role: u16,
    temperature_role: u16,
    motor_poles: usize,
    transmission: FixedTransmission,
    barrel_heater_w: f64,
    nozzle_heater_w: f64,
    heating_algorithm: HeatingAlgorithm,
}

const LAYOUT_V1: Layout = Layout {
    ek1100_role: 0,
    serial_role: 2,
    digital_out_role: 3,
    pressure_sensor_role: 4,
    temperature_role: 5,
    motor_poles: 4,
    transmission: FixedTransmission::new(1.0 / 34.0),
    barrel_heater_w: 700.0,
    nozzle_heater_w: 200.0,
    // V1 keeps its long-standing PID.
    heating_algorithm: HeatingAlgorithm::Pid,
};

// The generations are wired with different bands. These were once flattened to
// the V1 values when the two extruder modules were merged; keep them apart.
const LAYOUT_V2: Layout = Layout {
    ek1100_role: 0,
    serial_role: 1,
    digital_out_role: 2,
    pressure_sensor_role: 3,
    temperature_role: 4,
    motor_poles: 2,
    transmission: FixedTransmission::new(1.0 / 30.0),
    barrel_heater_w: 900.0,
    nozzle_heater_w: 150.0,
    // V2 has a calibrated thermal model the observer is tuned against.
    heating_algorithm: HeatingAlgorithm::ObserverPi,
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
            .can_execute(Self::can_extrude)
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
        // Cutout at 303 °C while targets stop at 300 °C: with the cutout at 300 °C a zone held at
        // 300 °C cannot heat the moment it crosses it, causing over- and undershooting.
        let max_temperature = ThermodynamicTemperature::new::<degree_celsius>(303.0);
        let max_target_temperature = ThermodynamicTemperature::new::<degree_celsius>(300.0);

        // The control law differs by hardware generation; the operator can switch it at runtime
        // through `heating.algorithm`.
        let mut init_zone = |zone: Zone, heating_element_wattage: f64| {
            TemperatureController::init(
                ctx,
                zone,
                TemperatureControllerConfig {
                    max_temperature,
                    max_target_temperature,
                    pwm_period: Duration::from_millis(500),
                    heating_element_wattage,
                    digital_port: zone.port(),
                    temperature_port: zone.port(),
                    strategy: build_strategy(layout.heating_algorithm, zone),
                },
            )
        };

        let temperature_controller_front = init_zone(Zone::Front, layout.barrel_heater_w)?;
        let temperature_controller_middle = init_zone(Zone::Middle, layout.barrel_heater_w)?;
        let temperature_controller_back = init_zone(Zone::Back, layout.barrel_heater_w)?;
        let temperature_controller_nozzle = init_zone(Zone::Nozzle, layout.nozzle_heater_w)?;

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

            heating_algorithm: ctx
                .config::<HeatingAlgorithm>("heating.algorithm")
                .default(layout.heating_algorithm)
                .on_external_changed(Self::on_heating_algorithm_changed)
                .build()?,

            // Shares the zones' target bound: a floor above what a zone may be heated to would
            // forbid extrusion permanently.
            min_extrusion_temperature: ctx
                .config::<degree_celsius>("extrusion.min_temperature")
                .default(DEFAULT_MIN_EXTRUSION_TEMPERATURE)
                .minimum(0.0)
                .maximum(max_target_temperature.get::<degree_celsius>())
                .build()?,

            mode: ctx.state::<Mode>("mode").build()?,

            combined_power: ctx.measurement::<watt>("power.combined").build()?,
            total_energy: ctx.measurement::<kilowatt_hour>("energy.total").build()?,

            last_energy_calculation_time: None,
            active_heating_algorithm: layout.heating_algorithm,
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The generations really are wired differently, and a refactor that
    /// flattens them back into one set of numbers is a regression — it already
    /// happened once, when the two extruder modules were merged.
    #[test]
    fn each_generation_keeps_its_own_heater_ratings() {
        assert_eq!(LAYOUT_V1.barrel_heater_w, 700.0);
        assert_eq!(LAYOUT_V1.nozzle_heater_w, 200.0);
        assert_eq!(LAYOUT_V2.barrel_heater_w, 900.0);
        assert_eq!(LAYOUT_V2.nozzle_heater_w, 150.0);
    }

    #[test]
    fn each_generation_keeps_its_default_algorithm() {
        assert_eq!(LAYOUT_V1.heating_algorithm, HeatingAlgorithm::Pid);
        assert_eq!(LAYOUT_V2.heating_algorithm, HeatingAlgorithm::ObserverPi);
    }
}
