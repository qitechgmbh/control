pub fn xor_u128_to_u64(value: u128) -> u64 {
    (value as u64) ^ ((value >> 64) as u64)
}

pub fn xor_u128_to_u32(value: u128) -> u32 {
    (value as u32) ^ ((value >> 32) as u32) ^ ((value >> 64) as u32) ^ ((value >> 96) as u32)
}

pub fn xor_u128_to_u16(value: u128) -> u16 {
    value
        .to_le_bytes()
        .iter()
        .fold(0, |buf, &byte| buf ^ byte as u16)
}

pub fn xor_u128_to_u8(value: u128) -> u8 {
    value.to_le_bytes().iter().fold(0, |buf, &byte| buf ^ byte)
}

// hash algorithm djb2
pub fn hashing(input_to_hash: &str) -> u128 {
    let mut hash: u128 = 5381;
    let chars = input_to_hash.chars();

    for ch in chars {
        hash = ((hash << 5) + hash) + (ch as u128);
    }
    hash
}
#[test]
fn test_hash() {
    let test_case = "/dev/ttyUSB1";
    let hash = hashing(test_case);
    let hash_2bytes = xor_u128_to_u16(hash);
    tracing::info!("{}", test_case);
    assert_eq!(test_case, "/dev/ttyUSB1");
    assert!(hash == 8977446972540388683742);
    assert!(hash_2bytes == 165);
    println!("{:?}", hash);
}
