quantity! {
    /// Pressure (base unit pascal, kg · m⁻¹ · s⁻²).
    quantity: Pressure; "pressure";
    /// Dimension of pressure, L⁻¹MT⁻² (base unit pascal, kg · m⁻¹ · s⁻²).
    dimension: ISQ<
        N1,  // length
        P1,  // mass
        N2,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @pascal: 1.0; "Pa", "pascal", "pascals";
        @bar: 1.0e5; "bar", "bar", "bar";
    }
}
