quantity! {
    /// Thermodynamic temperature (base unit kelvin, K).
    quantity: ThermodynamicTemperature; "thermodynamic temperature";
    /// Dimension of thermodynamic temperature, Th (base unit kelvin, K).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        Z0,  // time
        Z0,  // electric current
        P1,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @kelvin: 1.0; "K", "kelvin", "kelvins";
        @degree_celsius: 1.0_E0, 273.15_E0; "°C", "degree Celsius", "degrees Celsius";
    }
}
