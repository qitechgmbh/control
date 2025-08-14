/// Folds bytes using XOR to produce a u128 value.
///
/// Takes a slice of bytes and XORs them together in chunks to produce a u128.
/// If there are fewer than 16 bytes, the remaining bytes are padded with zeros.
pub fn byte_folding_u128(bytes: &[u8]) -> u128 {
    let mut result = 0u128;
    for (i, &byte) in bytes.iter().enumerate() {
        let shift = (i % 16) * 8;
        result ^= (byte as u128) << shift;
    }
    result
}

/// Folds bytes using XOR to produce a u64 value.
///
/// Takes a slice of bytes and XORs them together in chunks to produce a u64.
/// If there are fewer than 8 bytes, the remaining bytes are padded with zeros.
pub fn byte_folding_u64(bytes: &[u8]) -> u64 {
    let mut result = 0u64;
    for (i, &byte) in bytes.iter().enumerate() {
        let shift = (i % 8) * 8;
        result ^= (byte as u64) << shift;
    }
    result
}

/// Folds bytes using XOR to produce a u32 value.
///
/// Takes a slice of bytes and XORs them together in chunks to produce a u32.
/// If there are fewer than 4 bytes, the remaining bytes are padded with zeros.
pub fn byte_folding_u32(bytes: &[u8]) -> u32 {
    let mut result = 0u32;
    for (i, &byte) in bytes.iter().enumerate() {
        let shift = (i % 4) * 8;
        result ^= (byte as u32) << shift;
    }
    result
}

/// Folds bytes using XOR to produce a u16 value.
///
/// Takes a slice of bytes and XORs them together in chunks to produce a u16.
/// If there are fewer than 2 bytes, the remaining bytes are padded with zeros.
pub fn byte_folding_u16(bytes: &[u8]) -> u16 {
    let mut result = 0u16;
    for (i, &byte) in bytes.iter().enumerate() {
        let shift = (i % 2) * 8;
        result ^= (byte as u16) << shift;
    }
    result
}

/// Folds bytes using XOR to produce a u8 value.
///
/// Takes a slice of bytes and XORs them all together to produce a u8.
pub fn byte_folding_u8(bytes: &[u8]) -> u8 {
    bytes.iter().fold(0u8, |acc, &byte| acc ^ byte)
}

/// Computes a djb2 hash of the input bytes.
///
/// The djb2 algorithm is a simple non-cryptographic hash function created by
/// Daniel J. Bernstein. It uses the formula: `hash = hash * 33 + byte` for each byte.
///
/// This implementation uses the traditional 32-bit hash value and uses wrapping
/// arithmetic to handle potential overflows gracefully.
///
/// # Arguments
///
/// * `bytes` - A slice of bytes to hash
///
/// # Returns
///
/// A 32-bit hash value as a `u32`
///
/// # Examples
///
/// ```
/// use control_core::helpers::hashing::hash_djb2;
///
/// let hash = hash_djb2(b"hello world");
/// assert_ne!(hash, 0);
/// ```
///
/// # Algorithm Details
///
/// - Initial hash value: 5381 (djb2 magic number)
/// - For each byte: `hash = hash * 33 + byte`
/// - Uses wrapping arithmetic to prevent overflow panics
pub fn hash_djb2(bytes: &[u8]) -> u32 {
    let mut hash: u32 = 5381;

    for byte in bytes {
        hash = hash.wrapping_mul(33).wrapping_add(*byte as u32);
    }
    hash
}

#[test]
fn test_hash() {
    let test_case = "/dev/ttyUSB1";
    let hash_u32 = hash_djb2(test_case.as_bytes());
    let hash_u16 = byte_folding_u16(&hash_u32.to_le_bytes());
    tracing::info!("{}", test_case);
    assert_eq!(test_case, "/dev/ttyUSB1");
    assert_eq!(hash_u32, 266121182);
    assert_eq!(hash_u16, 40962);
}
