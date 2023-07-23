use bitvec::macros::internal::funty::Fundamental;
use bitvec::prelude::*;

use std::convert::From;

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

pub fn filled_bit_array<const N: usize>(bits: &BitSlice<u8, Msb0>) -> [u8; N] {
    let bit_start = N - bits.len();
    let mut new = [0; N];
    for i in 0..bits.len() {
        new[bit_start + i] = bits[i].as_u8();
    }
    new
}

#[cfg(test)]
mod tests {
    // use super::*;
    use bitvec::prelude::*;

    #[test]
    fn test_from_bits() {
        assert_eq!(super::from_bits::<u8>(&[0, 0, 0, 0, 0, 0, 0, 0]), Some(0));
        assert_eq!(super::from_bits::<u8>(&[0, 0, 0, 0, 0, 0, 0, 1]), Some(1));
        assert_eq!(super::from_bits::<u8>(&[0, 1, 0, 0, 0, 0, 0, 0]), Some(64));
    }

    #[test]
    fn test_filled_bit_array() {
        let slice = (&[16u8]).view_bits::<Msb0>();
        let filled = super::filled_bit_array::<8>(slice);
        assert_eq!(filled, [0, 0, 0, 1u8, 0, 0, 0, 0]);
    }
}
