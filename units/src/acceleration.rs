quantity! {
    /// Acceleration (base unit meter per second squared, m · s⁻²).
    quantity: Acceleration; "acceleration";
    /// Dimension of acceleration, LT⁻² (base unit meter per second squared, m · s⁻²).
    dimension: ISQ<
        P1,  // length
        Z0,  // mass
        N2,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @meter_per_second_squared: 1.0; "m/s²", "meter per second squared", "meters per second squared";
        @meter_per_minute_per_second: 0.016_666_666_666_666_67; "m/min/s", "meters per minute per second", "meters per minute per second";
    }
}
