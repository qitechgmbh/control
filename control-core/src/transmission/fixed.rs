use crate::transmission::Transmission;

#[derive(Debug, Clone)]
pub struct FixedTransmission {
    ratio: f64,
}

impl Transmission for FixedTransmission {
    fn get_ratio(&self) -> f64 {
        self.ratio
    }
}

impl FixedTransmission {
    /// Creates a new FixedTransmission with the specified ratio.
    ///
    /// # Arguments
    ///
    /// * `ratio` - The transmission ratio (output over input).
    ///
    /// # Returns
    ///
    /// A new instance of FixedTransmission.
    pub const fn new(ratio: f64) -> Self {
        Self { ratio }
    }
}
