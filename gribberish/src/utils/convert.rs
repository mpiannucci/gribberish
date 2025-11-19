
pub fn read_u16_from_bytes(data: &[u8], offset: usize) -> Option<u16> {
    match data[offset..offset + 2].try_into() {
        Ok(b) => Some(u16::from_be_bytes(b)),
        Err(_) => None,
    }
}

pub fn read_u24_from_bytes(data: &[u8], offset: usize) -> Option<u32> {
    if offset + 3 > data.len() {
        return None;
    }
    Some(
        ((data[offset] as u32) << 16)
            | ((data[offset + 1] as u32) << 8)
            | (data[offset + 2] as u32),
    )
}

pub fn read_u32_from_bytes(data: &[u8], offset: usize) -> Option<u32> {
    match data[offset..offset + 4].try_into() {
        Ok(b) => Some(u32::from_be_bytes(b)),
        Err(_) => None,
    }
}

pub fn read_u64_from_bytes(data: &[u8], offset: usize) -> Option<u64> {
    match data[offset..offset + 8].try_into() {
        Ok(b) => Some(u64::from_be_bytes(b)),
        Err(_) => None,
    }
}

pub fn read_f32_from_bytes(data: &[u8], offset: usize) -> Option<f32> {
    match data[offset..offset + 4].try_into() {
        Ok(b) => Some(f32::from_be_bytes(b)),
        Err(_) => None,
    }
}
