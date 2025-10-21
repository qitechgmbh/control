quantity! {
    /// Amount of substance (base unit mole, mol).
    quantity: AmountOfSubstance; "amount of substance";
    /// Dimension of amount of substance, N (base unit mole, mol).
    dimension: ISQ<
        Z0,  // length
        Z0,  // mass
        Z0,  // time
        Z0,  // electric current
        Z0,  // thermodynamic temperature
        P1,  // amount of substance
        Z0>; // luminous intensity
    units {
        @mole: 1.0; "mol", "mole", "moles";
    }
}
