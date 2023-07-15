use std::convert::From;
use std::vec::Vec;

pub fn read_u16_from_bytes(data: &[u8], offset: usize) -> Option<u16> {
    if data.len() < offset + 2 {
        return None;
    }

    let mut l: [u8; 2] = Default::default();
    l.copy_from_slice(&data[offset..offset + 2]);
    Some(u16::from_be_bytes(l))
}

pub fn read_u32_from_bytes(data: &[u8], offset: usize) -> Option<u32> {
    if data.len() < offset + 4 {
        return None;
    }

    let mut l: [u8; 4] = Default::default();
    l.copy_from_slice(&data[offset..offset + 4]);
    Some(u32::from_be_bytes(l))
}

pub fn read_u64_from_bytes(data: &[u8], offset: usize) -> Option<u64> {
    if data.len() < offset + 8 {
        return None;
    }

    let mut l: [u8; 8] = Default::default();
    l.copy_from_slice(&data[offset..offset + 8]);
    Some(u64::from_be_bytes(l))
}

pub fn read_f32_from_bytes(data: &[u8], offset: usize) -> Option<f32> {
    if data.len() < offset + 4 {
        return None;
    }

    let mut l: [u8; 4] = Default::default();
    l.copy_from_slice(&data[offset..offset + 4]);
    Some(f32::from_be_bytes(l))
}

pub fn from_bits<T>(bits: &[u8]) -> Option<T>
where
    T: num::Unsigned + From<u8>,
{
    if bits.len() != (std::mem::size_of::<T>() * 8) {
        return None;
    }

    let value = bits
        .iter()
        .fold(T::from(0), |acc, &b| acc * T::from(2) + T::from(b));

    Some(value)
}

pub fn filled_bit_array<const N: usize>(bits: &[u8]) -> [u8; N] {
    let bit_start = N - bits.len();
    let mut new = [0; N];
    for i in 0..bits.len() {
        new[bit_start + i] = bits[i];
    }
    new
}

pub fn bit_array_from_bytes(data: &[u8]) -> Vec<u8> {
    data.iter()
        .map(|r| format!("{:08b}", r))
        .flat_map(|s| {
            s.chars()
                .map(|c| c.to_digit(2).unwrap_or(0) as u8)
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>()
}

pub fn bits_to_bytes(bits: Vec<u8>) -> Option<Vec<u8>> {
    if bits.len() % 8 != 0 {
        return None;
    }

    let mut bytes = Vec::new();
    for i in (0..bits.len()).step_by(8) {
        let value = bits[i..i + 8].iter().fold(0u8, |acc, &b| acc * 2 + b);
        bytes.push(value);
    }
    Some(bytes)
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_from_bits() {
        assert_eq!(super::from_bits::<u8>(&[0, 0, 0, 0, 0, 0, 0, 0]), Some(0));
        assert_eq!(super::from_bits::<u8>(&[0, 0, 0, 0, 0, 0, 0, 1]), Some(1));
        assert_eq!(super::from_bits::<u8>(&[0, 1, 0, 0, 0, 0, 0, 0]), Some(64));
    }

    #[test]
    fn test_filled_bit_array() {
        assert_eq!(super::filled_bit_array::<8>(&[1, 0, 0, 0, 0]), [0, 0, 0, 1u8, 0, 0, 0, 0]);
    }

    #[test]
    fn test_bit_array_from_bytes() {
        assert_eq!(super::bit_array_from_bytes(&[128]), vec![1, 0, 0, 0, 0, 0, 0, 0]);
        assert_eq!(super::bit_array_from_bytes(&[128, 64, 4]), vec![
            1, 0, 0, 0, 0, 0, 0, 0,
            0, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0,
        ]);
    }

    #[test]
    fn test_bits_to_bytes() {
        assert_eq!(
            super::bits_to_bytes(vec![0, 0, 0, 0, 0, 0, 0, 0]),
            Some(vec![0])
        );
        assert_eq!(
            super::bits_to_bytes(vec![0, 0, 0, 0, 0, 0, 0, 1]),
            Some(vec![1])
        );
        assert_eq!(
            super::bits_to_bytes(vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
            Some(vec![64, 1])
        );
        assert_eq!(
            super::bits_to_bytes(vec![0, 1, 0, 0, 0, 0, 0, 0, 0, 1]),
            None
        );
    }
}
