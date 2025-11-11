use uom::Kind;

#[macro_use]
extern crate uom;

system! {
    quantities: ISQ {
        /// Length, one of the base quantities in the ISQ, denoted by the symbol L. The base unit
        /// for length is meter in the SI.
        length: meter, L;
        /// Mass, one of the base quantities in the ISQ, denoted by the symbol M. The base unit
        /// for mass is kilogram in the SI.
        mass: kilogram, M;
        /// Time, one of the base quantities in the ISQ, denoted by the symbol T. The base unit
        /// for time is second in the SI.
        time: second, T;
        /// Electric current, one of the base quantities in the ISQ, denoted by the symbol I. The
        /// base unit for electric current is ampere in the SI.
        electric_current: ampere, I;
        /// Thermodynamic temperature, one of the base quantities in the ISQ, denoted by the symbol
        /// Th (Θ). The base unit for thermodynamic temperature is kelvin in the SI.
        thermodynamic_temperature: kelvin, Th;
        /// Amount of substance, one of the base quantities in the ISQ, denoted by the symbol N.
        /// The base unit for amount of substance is mole in the SI.
        amount_of_substance: mole, N;
        /// Luminous intensity, one of the base quantities in the ISQ, denoted by the symbol J. The
        /// base unit for luminous intensity is candela in the SI.
        luminous_intensity: candela, J;
    }

    units: U {
        acceleration::Acceleration,
        amount_of_substance::AmountOfSubstance,
        angle::Angle,
        angular_acceleration::AngularAcceleration,
        angular_jerk::AngularJerk,
        angular_velocity::AngularVelocity,
        electric_current::ElectricCurrent,
        electric_potential::ElectricPotential,
        frequency::Frequency,
        jerk::Jerk,
        length::Length,
        luminous_intensity::LuminousIntensity,
        mass::Mass,
        pressure::Pressure,
        ratio::Ratio,
        thermodynamic_temperature::ThermodynamicTemperature,
        time::Time,
        velocity::Velocity,
        volume_rate::VolumeRate,
    }
}

pub trait AngleKind: Kind {}

pub mod f64 {
    mod mks {
        pub use super::super::*;
    }

    ISQ!(self::mks, f64);
}

pub use f64::*;
pub use uom::ConstZero;
