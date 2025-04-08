/// A 16-bit integer wrapper providing utilities for different integer representations.
///
/// This struct wraps a u16 value and provides methods to convert between different
/// integer representations (unsigned, two's complement signed, sign-magnitude).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Integer16 {
    raw: u16,
}

impl Integer16 {
    /// Creates a new `Integer16` from an unsigned 16-bit value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let value = Integer16::from_unsigned(42);
    /// assert_eq!(value.into_unsigned(), 42);
    /// ```
    #[inline]
    pub fn from_unsigned(value: u16) -> Self {
        Self { raw: value }
    }

    /// Creates a new `Integer16` from a signed 16-bit value using two's complement.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let value = Integer16::from_signed(-42);
    /// assert_eq!(value.into_signed(), -42);
    /// ```
    #[inline]
    pub fn from_signed(value: i16) -> Self {
        Self { raw: value as u16 }
    }

    /// Creates a new `Integer16` from a signed 16-bit value using sign-magnitude representation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let value = Integer16::from_signed_magnitude(-42);
    /// assert_eq!(value.into_signed_magnitude(), -42);
    /// ```
    #[inline]
    pub fn from_signed_magnitude(value: i16) -> Self {
        if value < 0 {
            Self {
                raw: ((-value) as u16) | 0x8000,
            }
        } else {
            Self { raw: value as u16 }
        }
    }

    /// Converts the value to an unsigned 16-bit integer.
    #[inline]
    pub fn into_unsigned(self) -> u16 {
        self.raw
    }

    /// Converts the value to a signed 16-bit integer using two's complement.
    #[inline]
    pub fn into_signed(self) -> i16 {
        self.raw as i16
    }

    /// Converts the value to a signed 16-bit integer using sign-magnitude representation.
    ///
    /// In sign-magnitude representation, the most significant bit represents the sign
    /// (0 for positive, 1 for negative) and the remaining bits represent the magnitude.
    #[inline]
    pub fn into_signed_magnitude(self) -> i16 {
        if self.raw & 0x8000 != 0 {
            -((self.raw & 0x7FFF) as i16)
        } else {
            self.raw as i16
        }
    }

    /// Returns the raw underlying value.
    #[inline]
    pub fn raw_value(&self) -> u16 {
        self.raw
    }

    /// Sets the sign bit (most significant bit).
    #[inline]
    pub fn set_sign_bit(&mut self) {
        self.raw |= 0x8000;
    }

    /// Clears the sign bit (most significant bit).
    #[inline]
    pub fn clear_sign_bit(&mut self) {
        self.raw &= 0x7FFF;
    }

    /// Checks if the sign bit is set.
    #[inline]
    pub fn is_sign_bit_set(&self) -> bool {
        (self.raw & 0x8000) != 0
    }
}

// Implement common traits for easier usage

impl From<u16> for Integer16 {
    fn from(value: u16) -> Self {
        Self::from_unsigned(value)
    }
}

impl From<i16> for Integer16 {
    fn from(value: i16) -> Self {
        Self::from_signed(value)
    }
}

impl From<Integer16> for u16 {
    fn from(value: Integer16) -> Self {
        value.into_unsigned()
    }
}

impl From<Integer16> for i16 {
    fn from(value: Integer16) -> Self {
        value.into_signed()
    }
}

impl std::fmt::Display for Integer16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Integer16 {{ unsigned: {} (0x{:04X}), signed: {} (0x{:04X}), sign-magnitude: {} (0x{:04X}) }}",
            self.into_unsigned(),
            self.into_unsigned(),
            self.into_signed(),
            self.raw,
            self.into_signed_magnitude(),
            self.raw
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsigned_conversion() {
        let value = Integer16::from_unsigned(42);
        assert_eq!(value.into_unsigned(), 42);
    }

    #[test]
    fn test_signed_conversion() {
        let value = Integer16::from_signed(-42);
        assert_eq!(value.into_signed(), -42);
    }

    #[test]
    fn test_sign_magnitude_conversion() {
        let value = Integer16::from_signed_magnitude(-42);
        assert_eq!(value.into_signed_magnitude(), -42);

        let value = Integer16::from_signed_magnitude(42);
        assert_eq!(value.into_signed_magnitude(), 42);
    }

    #[test]
    fn test_sign_bit_operations() {
        let mut value = Integer16::from_unsigned(42);
        assert!(!value.is_sign_bit_set());

        value.set_sign_bit();
        assert!(value.is_sign_bit_set());

        value.clear_sign_bit();
        assert!(!value.is_sign_bit_set());
    }
}
