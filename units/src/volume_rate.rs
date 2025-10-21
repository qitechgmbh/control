quantity! {
    /// Volume rate (base unit cubic meter per second, m³ · s⁻¹).
    quantity: VolumeRate; "volume rate";
    /// Dimension of volume rate, L³T⁻¹ (base unit cubic meter per second, m³ · s⁻¹).
    dimension: ISQ<
        P3,     // length
        Z0,     // mass
        N1,     // time
        Z0,     // electric current
        Z0,     // thermodynamic temperature
        Z0,     // amount of substance
        Z0>;    // luminous intensity
    units {
        @cubic_meter_per_second: 1.0; "m³/s", "cubic meter per second", "cubic meters per second";
        @liter_per_second: 1.0e-3; "L/s", "liter per second", "liters per second";
        @liter_per_minute: 1.0e-3 / 6.0_E1; "L/min", "liter per minute", "liters per minute";
    }
}
