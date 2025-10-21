quantity! {
    /// Jerk (base unit meter per second cubed, m · s⁻³).
    quantity: Jerk; "jerk";
    /// Dimension of jerk, LT⁻³ (base unit meter per second cubed, m · s⁻³).
    dimension: ISQ<
        P1,  // length
        Z0,  // mass
        N3,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @meter_per_second_cubed: 1.0; "m/s³", "meter per second cubed", "meters per second cubed";
        @meter_per_minute_per_second_squared: 0.016_666_666_666_666_666_67; "m/min/s²", "meters per minute per second squared", "meters per minute per second squared";
    }
}
