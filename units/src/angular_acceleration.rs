use super::AngleKind;

quantity! {
    /// Angular acceleration (base unit radian per second squared, s⁻²).
    quantity: AngularAcceleration; "angular acceleration";
    /// Dimension of angular acceleration, T⁻² (base unit radian per second squared, s⁻²).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        N2,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    kind: dyn AngleKind;
    units {
        /// Derived unit of angular acceleration.
        @radian_per_second_squared: 1.0; "rad/s²", "radian per second squared", "radians per second squared";
        @degree_per_second_squared: 1.745_329_251_994_329_5_E-2; "°/s²", "degree per second squared", "degrees per second squared";
        @revolution_per_minute_per_second: 0.104_719_755_119_659_77; "rev/min/s", "revolutions per minute per second", "revolutions per minute per second";
    }
}
