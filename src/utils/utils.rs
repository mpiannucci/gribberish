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

// pub fn read_f64_from_bytes(data: &[u8], offset: usize) -> Option<f64> {
//     if data.len() < offset + 8 {
//         return None;
//     }

//     let mut l: [u8; 8] = Default::default();
//     l.copy_from_slice(&data[offset..offset + 8]);
//     Some(f64::from_be_bytes(l))
// }

pub fn from_bits<T>(bits: &[u8]) -> Option<T>
where
    T: num::Unsigned + From<u8>,
{
    if bits.len() != (std::mem::size_of::<T>() * 8) {
        return None;
    }

    let value = bits.iter()
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

// pub fn byte_to_bits(data: &u8) -> [u8; 8] {
//     let mut result = [0; 8];
//     let bit_string = format!("{:08b}", data);
//     for (i, b) in bit_string.char_indices() {
//         result[i] = b.to_digit(2).unwrap_or(0) as u8;
//     }

//     result
// }

// pub fn positive_bit_count(data: &u8) -> u8 {
//     let bits = byte_to_bits(data);
//     bits.iter().sum()
// }

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

// https://github.com/erdc/grib_api/blob/master/src/grib_scaling.c
pub fn grib_power(s: i32, n: i32) -> f64 {
    let mut divisor = 1.0;
    let mut ss = s;

    while ss < 0 {
        divisor /= n as f64;
        ss += 1;
    }

    while ss > 0 {
        divisor *= n as f64;
        ss -= 1;
    }

    return divisor;
}

#[cfg(test)]
mod tests {
    // use super::*;


    #[test]
    fn test_convert_signed() {
        let neg_one: u8 = 0b10000001;
        assert_eq!(as_signed!(neg_one, i8), -1);

        let four: u8 = 0b00000100;
        assert_eq!(as_signed!(four, i8), 4);
    }
}
