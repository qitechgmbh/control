quantity! {
    /// Electric current (base unit ampere, A).
    quantity: ElectricCurrent; "electric current";
    /// Dimension of electric current, I (base unit ampere, A).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        Z0,  // time
        P1,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @milliampere: 1.0e-3; "mA", "millampere", "millamperes";
        @centiampere: 1.0e-2; "cA", "centiampere", "centiamperes";
        @ampere: 1.0; "A", "ampere", "amperes";
    }
}
