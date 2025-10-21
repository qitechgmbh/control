use super::AngleKind;

quantity! {
    /// Angular velocity (base unit radian per second, s⁻¹).
    quantity: AngularVelocity; "angular velocity";
    /// Dimension of angular velocity, T⁻¹ (base unit radian per second, s⁻¹).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        N1,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    kind: dyn AngleKind;
    units {
        @radian_per_second: 1.0; "rad/s", "radian per second", "radians per second";
        @degree_per_second: 1.745_329_251_994_329_5_E-2; "°/s", "degree per second", "degrees per second";
        @revolution_per_second: 6.283_185_307_179_586; "rps", "revolution per second", "revolutions per second";
        @revolution_per_minute: 1.047_197_551_196_597_7e-1; "rpm", "revolution per minute", "revolutions per minute";
    }
}
