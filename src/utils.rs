use std::vec::Vec;
use std::convert::From;
use crate::num;

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

pub fn read_i16_from_bytes(data: &[u8], offset: usize) -> Option<i16> {
    if data.len() < offset + 2 {
        return None;
    }

    let mut l: [u8; 2] = Default::default();
    l.copy_from_slice(&data[offset..offset + 2]);
    Some(i16::from_be_bytes(l))
}

pub fn read_f32_from_bytes(data: &[u8], offset: usize) -> Option<f32> {
        if data.len() < offset + 4 {
        return None;
    }

    let mut l: [u8; 4] = Default::default();
    l.copy_from_slice(&data[offset..offset + 4]);
    Some(f32::from_be_bytes(l))
}

pub fn read_f64_from_bytes(data: &[u8], offset: usize) -> Option<f64> {
    if data.len() < offset + 8 {
        return None;
    }

    let mut l: [u8; 8] = Default::default();
    l.copy_from_slice(&data[offset..offset + 8]);
    Some(f64::from_be_bytes(l))
}

pub fn from_bits<T>(bits: &[u8]) -> Option<T> where T: num::integer::Integer + From<u8> {
    if bits.len() != (std::mem::size_of::<T>() * 8) {
        return None;
    }
    
    Some(bits.iter().rev().fold(T::from(0), |acc, &b| acc*T::from(2) + T::from(b)))
}

pub fn bit_array_from_bytes(data: &[u8]) -> Vec<u8> {
    data            
        .iter()
        .map(|r| format!("{:b}", r))
        .flat_map(|s| s.chars()
                                .map(|c| c.to_digit(10).unwrap_or(0) as u8)
                                .collect::<Vec<u8>>())
        .collect::<Vec<u8>>()
}