#[macro_use]
mod length;
#[macro_use]
mod time;
#[macro_use]
mod mass;
#[macro_use]
mod electric_current;
#[macro_use]
mod thermodynamic_temperature;
#[macro_use]
mod amount_of_substance;
#[macro_use]
mod luminous_intensity;

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
        mod length::Length,
        mod mass::Mass,
        mod time::Time,
        mod electric_current::ElectricCurrent,
        mod thermodynamic_temperature::ThermodynamicTemperature,
        mod amount_of_substance::AmountOfSubstance,
        mod luminous_intensity::LuminousIntensity,
    }
}

mod f64 {
    mod mks {
        pub use super::super::*;
    }

    ISQ!(self::mks, f64);
}
