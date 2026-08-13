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

/// Read a two's complement signed 32-bit value.
///
/// Most signed GRIB fields use sign-magnitude (see the `as_signed` macro), but the
/// forecast time in section 4 is written in two's complement by NCEP when the
/// statistical time interval starts before the reference time.
pub fn read_i32_from_bytes(data: &[u8], offset: usize) -> Option<i32> {
    match data.get(offset..offset + 4)?.try_into() {
        Ok(b) => Some(i32::from_be_bytes(b)),
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

#[cfg(test)]
mod tests {
    use super::read_i32_from_bytes;

    #[test]
    fn reads_twos_complement_i32() {
        // NCEP writes negative GRIB2 forecast times in two's complement:
        // 0xfffffff9 is the -7 hour interval start in NAQFC daily maximum products.
        assert_eq!(read_i32_from_bytes(&[0xff, 0xff, 0xff, 0xf9], 0), Some(-7));
        assert_eq!(read_i32_from_bytes(&[0x00, 0x00, 0x00, 0x17], 0), Some(23));
        assert_eq!(read_i32_from_bytes(&[0x00, 0x00, 0x00, 0x00], 0), Some(0));
    }

    #[test]
    fn returns_none_when_out_of_bounds() {
        assert_eq!(read_i32_from_bytes(&[0x00, 0x00, 0x00], 0), None);
        assert_eq!(read_i32_from_bytes(&[0x00, 0x00, 0x00, 0x01], 1), None);
    }
}
