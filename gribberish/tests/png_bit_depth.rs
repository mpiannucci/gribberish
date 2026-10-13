use bitvec::prelude::*;
use gribberish::message::read_messages;
use gribberish::sections::data_representation::DataRepresentationSection;

// 40 × 50 sub-area cut with eccodes from a real MRMS MultiSensor QPE 01H message
// (2026-10-13 06Z) whose PNG was written at 8 bits because the hour's value range fit a
// byte; most MRMS hours are 16-bit. DRT 5.41, reference 7, binary scale 0, decimal scale 1.
// Expected values are eccodes' decode of the cut file.
const MRMS_8BIT: &[u8] = include_bytes!("../../test-data/mrms_qpe01h_png8_subarea.grib2");

/// Section 5 for DRT 5.41: reference -30, binary scale 0, decimal scale 1.
fn png_section(count: u32, bit_count: u8) -> Vec<u8> {
    let mut section = vec![0u8; 21];
    section[..4].copy_from_slice(&21u32.to_be_bytes());
    section[4] = 5;
    section[5..9].copy_from_slice(&count.to_be_bytes());
    section[9..11].copy_from_slice(&41u16.to_be_bytes());
    section[11..15].copy_from_slice(&(-30.0f32).to_be_bytes());
    section[17..19].copy_from_slice(&1u16.to_be_bytes());
    section[19] = bit_count;
    section
}

fn unpack_png(depth: png::BitDepth, bit_count: u8, data: &[u8]) -> Vec<f64> {
    let mut payload = Vec::new();
    let mut encoder = png::Encoder::new(&mut payload, 4, 3);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(depth);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(data)
        .unwrap();

    let bits = BitSlice::<u8, Msb0>::from_slice(&payload);
    DataRepresentationSection::from_data(&png_section(12, bit_count))
        .data_representation_template()
        .unwrap()
        .unpack(bits)
        .unwrap()
}

fn assert_values(values: &[f64], raw: impl Iterator<Item = u32>) {
    assert_eq!(values.len(), 12);
    for (value, raw) in values.iter().zip(raw) {
        let expected = (raw as f64 - 30.0) / 10.0;
        assert!((value - expected).abs() < 1e-9, "{value} != {expected}");
    }
}

#[test]
fn png_unpacks_one_point_per_sample_at_8_and_16_bits() {
    let raw8: [u8; 12] = [30, 31, 32, 33, 0, 255, 30, 60, 90, 120, 150, 180];
    let values = unpack_png(png::BitDepth::Eight, 8, &raw8);
    assert_values(&values, raw8.iter().map(|&r| r as u32));

    let raw16: [u16; 12] = [30, 34, 676, 65535, 0, 1, 256, 257, 30, 30, 30, 30];
    let bytes: Vec<u8> = raw16.iter().flat_map(|v| v.to_be_bytes()).collect();
    let values = unpack_png(png::BitDepth::Sixteen, 16, &bytes);
    assert_values(&values, raw16.iter().map(|&r| r as u32));
}

#[test]
fn read_real_8bit_png_mrms_message() {
    let message = read_messages(MRMS_8BIT).next().unwrap();
    assert_eq!(message.grid_dimensions().unwrap(), (40, 50));
    let values = message.data().unwrap();
    assert_eq!(values.len(), 40 * 50);
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    for (actual, expected) in [
        (values[0], 4.2),
        (values[1999], 2.5),
        (min, 0.7),
        (max, 21.9),
        (mean, 5.74365),
    ] {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }
}
