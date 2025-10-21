use crate::AngleKind;

quantity! {
    /// Angular jerk (base unit radian per second cubed, s⁻³).
    quantity: AngularJerk; "angular jerk";
    /// Dimension of angular jerk, T⁻³ (base unit radian per second cubed, s⁻³).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        N3,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    kind: dyn AngleKind;
    units {
        @radian_per_second_cubed: 1.0; "rad/s³", "radian per second cubed", "radians per second cubed";
        @degree_per_second_cubed: 1.745_329_251_994_329_5e-2; "°/s³", "degree per second cubed", "degrees per second cubed";
        @revolution_per_minute_per_second_squared: 0.104_719_755_119_659_77; "rev/min/s²", "revolution per minute per second squared", "revolution per minute per second squared";
    }
}
