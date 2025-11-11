quantity! {
    /// Velocity (base unit meter per second, m · s⁻¹).
    quantity: Velocity; "velocity";
    /// Dimension of velocity, LT⁻¹ (base unit meter per second, m · s⁻¹).
    dimension: ISQ<
        P1,  // length
        Z0,  // mass
        N1,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @millimeter_per_second: 1.0e-3; "mm/s", "millimeter per second", "millimeters per second";
        @meter_per_second: 1.0; "m/s", "meter per second", "meters per second";
        @meter_per_minute: 0.016_666_666_666_666_67; "m/min", "meters per minute", "meters per minute";
    }
}
