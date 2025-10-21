quantity! {
    /// Length (base unit meter, m).
    quantity: Length; "length";
    /// Length dimension, m.
    dimension: ISQ<
        P1,  // length
        Z0,  // mass
        Z0,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @millimeter: 1.0e-3; "mm", "millimeter", "millimeters";
        @centimeter: 1.0e-2; "cm", "centimeter", "centimeters";
        @meter: 1.0; "m", "meter", "meters";
    }
}
