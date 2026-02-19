pub fn read_u16_from_bytes(data: &[u8], offset: usize) -> Option<u16> {
    match data[offset..offset + 2].try_into() {
        Ok(b) => Some(u16::from_be_bytes(b)),
        Err(_) => None,
    }
}

/// Read a GRIB1 signed 16-bit value encoded as sign-magnitude.
/// Bit 1 is sign and bits 2-16 are magnitude (WMO GRIB1 convention).
pub fn read_grib1_sign_magnitude_i16_from_bytes(data: &[u8], offset: usize) -> Option<i16> {
    let raw = read_u16_from_bytes(data, offset)?;
    let sign = (raw & 0x8000) != 0;
    let magnitude = (raw & 0x7FFF) as i16;
    Some(if sign { -magnitude } else { magnitude })
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

/// Read an IBM floating point value (used in GRIB1)
/// IBM float format: (-1)^sign * 0.mantissa * 16^(exponent - 64)
pub fn read_ibm_f32_from_bytes(data: &[u8], offset: usize) -> Option<f32> {
    if offset + 4 > data.len() {
        return None;
    }

    let bytes = &data[offset..offset + 4];

    // Extract sign bit (bit 0)
    let sign = (bytes[0] & 0x80) != 0;

    // Extract exponent (bits 1-7)
    let exponent = (bytes[0] & 0x7F) as i32;

    // Extract mantissa (bits 8-31)
    let mantissa = ((bytes[1] as u32) << 16) | ((bytes[2] as u32) << 8) | (bytes[3] as u32);

    // Handle zero
    if mantissa == 0 {
        return Some(0.0);
    }

    // Convert to IEEE float
    // IBM: value = mantissa * 16^(exponent - 64) / 2^24
    // Simplify: 16^(exponent - 64) = 2^(4 * (exponent - 64))
    let power = 4 * (exponent - 64);
    let mut value = (mantissa as f64) * 2_f64.powi(power - 24);

    if sign {
        value = -value;
    }

    Some(value as f32)
}
