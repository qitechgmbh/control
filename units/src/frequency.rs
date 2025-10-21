quantity! {
    /// Frequency (base unit hertz, s⁻¹).
    quantity: Frequency; "frequency";
    /// Dimension of frequency, T⁻¹ (base unit hertz, s⁻¹).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        N1,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @millihertz: 1.0e-3; "mHz", "millihertz", "millihertz";
        @centihertz: 1.0e-2; "cHz", "centihertz", "centihertz";
        @hertz: 1.0; "Hz", "hertz", "hertz";
        @cycle_per_minute: 1.666_666_666_666_666_6e-2; "1/min", "cycle per minute", "cycles per minute";
    }
}
