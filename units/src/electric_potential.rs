quantity! {
    /// Electric potential (base unit volt, m² · kg · s⁻³ · A⁻¹).
    quantity: ElectricPotential; "electric potential";
    /// Dimension of electric potential, L²MT⁻³I⁻¹ (base unit volt, m² · kg · s⁻³ · A⁻¹).
    dimension: ISQ<
        P2,  // length
        P1,  // mass
        N3,  // time
        N1,  // electric current
        Z0,  // thermodynamic temperature
        Z0,  // amount of substance
        Z0>; // luminous intensity
    units {
        @millivolt: 1.0e-3; "mV", "millivolt", "millivolts";
        @centivolt: 1.0e-2; "cV", "centivolt", "centivolts";
        @volt: 1.0; "V", "volt", "volts";
    }
}
