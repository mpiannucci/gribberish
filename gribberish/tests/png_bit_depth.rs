use bitvec::prelude::*;
use gribberish::message::read_messages;
use gribberish::sections::data_representation::DataRepresentationSection;

// MRMS MultiSensor QPE 01H Pass 1 from the public NOAA MRMS feed. DRT 5.41 (PNG) with an
// 8-bit greyscale image, reference value -30, binary scale 0, decimal scale 1, no bitmap.
// The grid is 3500 × 7000. Most MRMS hours are written as 16-bit PNGs; this hour had a
// small enough value range that the encoder chose 8 bits.
const MRMS_8BIT: &[u8] =
    include_bytes!("../../test-data/MRMS_MultiSensor_QPE_01H_Pass1_00.00_20261013-060000.grib2");

fn signed_scale(value: i16) -> [u8; 2] {
    (value.unsigned_abs() | if value < 0 { 0x8000 } else { 0 }).to_be_bytes()
}

/// Section 5 for DRT 5.41 with the MRMS scaling (reference -30, binary 0, decimal 1).
fn png_section(count: u32, bit_count: u8) -> Vec<u8> {
    let mut section = vec![0u8; 21];
    section[..4].copy_from_slice(&21u32.to_be_bytes());
    section[4] = 5;
    section[5..9].copy_from_slice(&count.to_be_bytes());
    section[9..11].copy_from_slice(&41u16.to_be_bytes());
    section[11..15].copy_from_slice(&(-30.0f32).to_be_bytes());
    section[15..17].copy_from_slice(&signed_scale(0));
    section[17..19].copy_from_slice(&signed_scale(1));
    section[19] = bit_count;
    section[20] = 0;
    section
}

fn encode_png(width: u32, height: u32, depth: png::BitDepth, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(depth);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(data).unwrap();
    }
    out
}

fn unpack(section: &[u8], payload: &[u8]) -> Vec<f64> {
    let bits = BitSlice::<u8, Msb0>::from_slice(payload);
    DataRepresentationSection::from_data(section)
        .data_representation_template()
        .unwrap()
        .unpack(bits)
        .unwrap()
}

#[test]
fn png_8bit_greyscale_unpacks_one_byte_per_point() {
    // 4 × 3 raw values; (raw - 30) / 10 after scaling.
    let raw: [u8; 12] = [30, 31, 32, 33, 0, 255, 30, 60, 90, 120, 150, 180];
    let payload = encode_png(4, 3, png::BitDepth::Eight, &raw);
    let values = unpack(&png_section(12, 8), &payload);
    let expected: Vec<f64> = raw.iter().map(|&r| (r as f64 - 30.0) / 10.0).collect();
    assert_eq!(values.len(), 12);
    for (value, expected) in values.iter().zip(expected) {
        assert!((value - expected).abs() < 1e-9, "{value} != {expected}");
    }
}

#[test]
fn png_16bit_greyscale_unpacks_two_bytes_per_point() {
    let raw: [u16; 12] = [30, 34, 676, 65535, 0, 1, 256, 257, 30, 30, 30, 30];
    let bytes: Vec<u8> = raw.iter().flat_map(|v| v.to_be_bytes()).collect();
    let payload = encode_png(4, 3, png::BitDepth::Sixteen, &bytes);
    let values = unpack(&png_section(12, 16), &payload);
    let expected: Vec<f64> = raw.iter().map(|&r| (r as f64 - 30.0) / 10.0).collect();
    assert_eq!(values.len(), 12);
    for (value, expected) in values.iter().zip(expected) {
        assert!((value - expected).abs() < 1e-9, "{value} != {expected}");
    }
}

#[test]
fn read_real_8bit_png_mrms_message() {
    // Expected values come from decoding Section 7 with Pillow and applying
    // (raw - 30) / 10 by hand.
    let messages: Vec<_> = read_messages(MRMS_8BIT).collect();
    assert_eq!(messages.len(), 1);
    let message = &messages[0];
    assert_eq!(message.grid_dimensions().unwrap(), (3500, 7000));
    assert!(!message.has_bitmap());
    let values = message.data().unwrap();
    assert_eq!(values.len(), 3500 * 7000);
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!((min - -3.0).abs() < 1e-9, "min {min}");
    assert!((max - 21.9).abs() < 1e-9, "max {max}");
    assert!((values[0] - 0.0).abs() < 1e-9);
    assert!((values[12_345_678] - 0.0).abs() < 1e-9);
    assert!((values[values.len() - 1] - -3.0).abs() < 1e-9);
}
