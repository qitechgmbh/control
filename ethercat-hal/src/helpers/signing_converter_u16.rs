/// A 16-bit integer wrapper providing utilities for different integer representations.
///
/// This struct wraps a u16 value and provides methods to convert between different
/// integer representations (unsigned, two's complement signed, sign-magnitude).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U16SigningConverter {
    pub raw: u16,
}

impl U16SigningConverter {
    /// Creates a new `Integer16` from an raw 16-bit value.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let value = Integer16::from_raw(42);
    /// assert_eq!(value.into_unsigned(), 42);
    /// ```
    #[inline]
    pub const fn load_raw(value: u16) -> Self {
        Self { raw: value }
    }

    /// Converts the value to an unsigned 16-bit integer.
    #[inline]
    pub const fn as_unsigned(self) -> u16 {
        self.raw
    }

    /// Converts the value to a signed 16-bit integer using two's complement.
    #[inline]
    pub const fn as_signed(self) -> i16 {
        self.raw as i16
    }

    /// Converts the value to a signed 16-bit integer using sign-magnitude representation.
    ///
    /// In sign-magnitude representation, the most significant bit represents the sign
    /// (0 for positive, 1 for negative) and the remaining bits represent the magnitude.
    #[inline]
    pub const fn as_signed_magnitude(self) -> i16 {
        if self.raw & 0x8000 != 0 {
            -((self.raw & 0x7FFF) as i16)
        } else {
            self.raw as i16
        }
    }

    #[inline]
    pub const fn as_absolute(self) -> i16 {
        self.as_signed().abs()
    }
}

// Implement common traits for easier usage

impl From<u16> for U16SigningConverter {
    fn from(value: u16) -> Self {
        Self::load_raw(value)
    }
}

impl From<U16SigningConverter> for u16 {
    fn from(value: U16SigningConverter) -> Self {
        value.as_unsigned()
    }
}

impl std::fmt::Display for U16SigningConverter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "U16SigningConverter {{ unsigned: {} (0x{:04X}), signed: {} (0x{:04X}), sign-magnitude: {} (0x{:04X}) }}",
            self.as_unsigned(),
            self.as_unsigned(),
            self.as_signed(),
            self.raw,
            self.as_signed_magnitude(),
            self.raw
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_as_signed() {
        let value = U16SigningConverter::load_raw(0x7FFF);
        assert_eq!(value.as_signed(), 32767i16);

        let value = U16SigningConverter::load_raw(0x3FFF);
        assert_eq!(value.as_signed(), 16383i16);

        let value = U16SigningConverter::load_raw(0x0001);
        assert_eq!(value.as_signed(), 1i16);

        let value = U16SigningConverter::load_raw(0x0000);
        assert_eq!(value.as_signed(), 0i16);

        let value = U16SigningConverter::load_raw(0xFFFF);
        assert_eq!(value.as_signed(), -1i16);

        let value = U16SigningConverter::load_raw(0xC001);
        assert_eq!(value.as_signed(), -16383i16);

        let value = U16SigningConverter::load_raw(0x8000);
        assert_eq!(value.as_signed(), -32768i16);
    }

    #[test]
    fn test_as_signed_magnitude() {
        let value = U16SigningConverter::load_raw(0x7FFF);
        assert_eq!(value.as_signed_magnitude(), 32767i16);

        let value = U16SigningConverter::load_raw(0x3FFF);
        assert_eq!(value.as_signed_magnitude(), 16383i16);

        let value = U16SigningConverter::load_raw(0x0001);
        assert_eq!(value.as_signed_magnitude(), 1i16);

        let value = U16SigningConverter::load_raw(0x0000);
        assert_eq!(value.as_signed_magnitude(), 0i16);

        let value = U16SigningConverter::load_raw(0x8001);
        assert_eq!(value.as_signed_magnitude(), -1i16);

        let value = U16SigningConverter::load_raw(0xBFFF);
        assert_eq!(value.as_signed_magnitude(), -16383i16);

        let value = U16SigningConverter::load_raw(0xFFFF);
        assert_eq!(value.as_signed_magnitude(), -32767i16);
    }

    #[test]
    fn test_as_unsigned() {
        let value = U16SigningConverter::load_raw(0x7FFF);
        assert_eq!(value.as_unsigned(), 32767u16);

        let value = U16SigningConverter::load_raw(0x3FFF);
        assert_eq!(value.as_unsigned(), 16383u16);

        let value = U16SigningConverter::load_raw(0x0001);
        assert_eq!(value.as_unsigned(), 1u16);

        let value = U16SigningConverter::load_raw(0x0000);
        assert_eq!(value.as_unsigned(), 0u16);

        let value = U16SigningConverter::load_raw(0x0001);
        assert_eq!(value.as_unsigned(), 1u16);

        let value = U16SigningConverter::load_raw(0x3FFF);
        assert_eq!(value.as_unsigned(), 16383u16);

        let value = U16SigningConverter::load_raw(0x7FFF);
        assert_eq!(value.as_unsigned(), 32767u16);
    }
}
