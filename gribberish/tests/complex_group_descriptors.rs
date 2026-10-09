use bitvec::prelude::*;
use gribberish::message::read_messages;
use gribberish::sections::bitmap::BitmapSection;
use gribberish::sections::data_representation::DataRepresentationSection;

// NOAA RRFS, 2026-10-09 12Z, f084 PEVAP at the surface. Sections 1,
// 4, 5 and 7 are retained verbatim; only grid dimensions/count and the bitmap
// were reduced to eight cells. wgrib2 independently decodes the three defined
// cells as 57700, 54500, 54500, matching the full source message.
const PEVAP: &[u8] =
    include_bytes!("../../test-data/rrfs.t12z.f084-PEVAP-complex-descriptors-tiny.grib2");

fn sections(message: &[u8]) -> Vec<Vec<u8>> {
    assert_eq!(&message[..4], b"GRIB");
    assert_eq!(
        u64::from_be_bytes(message[8..16].try_into().unwrap()) as usize,
        message.len()
    );
    assert_eq!(&message[message.len() - 4..], b"7777");
    let mut offset = 16;
    let mut result = Vec::new();
    while offset < message.len() - 4 {
        let length = u32::from_be_bytes(message[offset..offset + 4].try_into().unwrap()) as usize;
        result.push(message[offset..offset + length].to_vec());
        offset += length;
    }
    assert_eq!(offset, message.len() - 4);
    result
}

fn complex_section(template: u16, order: u8, count: u32, groups: u32) -> Vec<u8> {
    let length = if template == 3 { 49 } else { 47 };
    let mut section = vec![0; length];
    section[..4].copy_from_slice(&(length as u32).to_be_bytes());
    section[4] = 5;
    section[5..9].copy_from_slice(&count.to_be_bytes());
    section[9..11].copy_from_slice(&template.to_be_bytes());
    section[21] = 1;
    section[31..35].copy_from_slice(&groups.to_be_bytes());
    section[41] = 1;
    if template == 3 {
        section[47] = order;
        section[48] = 1;
    }
    section
}

fn append_fields(bits: &mut BitVec<u8, Msb0>, width: u8, values: &[u32]) {
    for &value in values {
        assert!(width != 0 || value == 0);
        assert!(width == 32 || value < (1u32 << width));
        for bit in (0..width).rev() {
            bits.push(value & (1 << bit) != 0);
        }
    }
}

fn pad_octet(bits: &mut BitVec<u8, Msb0>) {
    while bits.len() % 8 != 0 {
        bits.push(false);
    }
}

fn packed_data(spatial: &[u8], descriptors: &[(u8, &[u32])], groups: &[(u8, &[u32])]) -> Vec<u8> {
    let mut bits = BitVec::<u8, Msb0>::from_vec(spatial.to_vec());
    // Each descriptor array is octet-aligned, but the group data is contiguous.
    for &(width, values) in descriptors {
        append_fields(&mut bits, width, values);
        pad_octet(&mut bits);
    }
    for &(width, values) in groups {
        append_fields(&mut bits, width, values);
    }
    pad_octet(&mut bits);
    bits.into_vec()
}

fn unpack(section: &[u8], packed: &[u8]) -> Vec<f64> {
    DataRepresentationSection::from_data(section)
        .data_representation_template()
        .unwrap()
        .unpack(packed.view_bits::<Msb0>())
        .unwrap()
}

fn message(section5: &[u8], packed: &[u8], cell_count: u32, bitmap: &[u8]) -> Vec<u8> {
    let mut result = PEVAP[..16].to_vec();
    for mut section in sections(PEVAP) {
        match section[4] {
            3 => {
                section[6..10].copy_from_slice(&cell_count.to_be_bytes());
                section[30..34].copy_from_slice(&cell_count.to_be_bytes());
                section[34..38].copy_from_slice(&1u32.to_be_bytes());
            }
            5 => section = section5.to_vec(),
            6 => {
                section = vec![0, 0, 0, 0, 6, if bitmap.is_empty() { 255 } else { 0 }];
                section.extend(bitmap);
                let length = section.len() as u32;
                section[..4].copy_from_slice(&length.to_be_bytes());
            }
            7 => {
                section = vec![0, 0, 0, 0, 7];
                section.extend(packed);
                let length = section.len() as u32;
                section[..4].copy_from_slice(&length.to_be_bytes());
            }
            _ => {}
        }
        result.extend(section);
    }
    result.extend(b"7777");
    let length = result.len() as u64;
    result[8..16].copy_from_slice(&length.to_be_bytes());
    result
}

fn assert_values(actual: &[f64], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (index, (&actual, &expected)) in actual.iter().zip(expected).enumerate() {
        assert!(
            actual == expected || (actual.is_nan() && expected.is_nan()),
            "cell {index}: actual {actual}, expected {expected}"
        );
    }
}

#[test]
fn real_pevap_zero_bit_descriptors_preserve_scaled_values_and_bitmap() {
    let parts = sections(PEVAP);
    let section5 = parts.iter().find(|s| s[4] == 5).unwrap();
    // R=545, E=0, D=-2; no group-reference bits; one six-bit-wide
    // group with length reference 3 and explicit last length 3.
    assert_eq!(&section5[5..11], &[0, 0, 0, 3, 0, 2]);
    assert_eq!(
        f32::from_be_bytes(section5[11..15].try_into().unwrap()),
        545.0
    );
    assert_eq!(&section5[15..23], &[0, 0, 0x80, 2, 0, 0, 1, 0]);
    assert_eq!(
        &section5[31..47],
        &[0, 0, 0, 1, 6, 0, 0, 0, 0, 3, 1, 0, 0, 0, 3, 0]
    );
    let payload = &parts.iter().find(|s| s[4] == 7).unwrap()[5..];
    assert_eq!(payload, &[0x80, 0, 0]); // Six-bit integers 32, 0, 0.
    assert_eq!(unpack(section5, payload), vec![57700.0, 54500.0, 54500.0]);
    assert_eq!(
        &parts.iter().find(|s| s[4] == 6).unwrap()[5..],
        &[0, 0b01001010]
    );
    let messages: Vec<_> = read_messages(PEVAP).collect();
    assert_eq!(messages.len(), 1);
    let decoded = &messages[0];
    assert_eq!(decoded.grid_dimensions().unwrap(), (2, 4));
    assert_eq!(decoded.data_point_count().unwrap(), 3);
    assert!(decoded.has_bitmap());
    assert_values(
        &decoded.data().unwrap(),
        &[
            f64::NAN,
            57700.0,
            f64::NAN,
            f64::NAN,
            54500.0,
            f64::NAN,
            54500.0,
            f64::NAN,
        ],
    );
}

#[test]
fn nonzero_group_reference_bits_precede_implicit_descriptors() {
    let mut section = complex_section(2, 0, 3, 1);
    section[11..15].copy_from_slice(&100f32.to_be_bytes());
    section[15..17].copy_from_slice(&1u16.to_be_bytes());
    section[19] = 5;
    section[35] = 3;
    section[37..41].copy_from_slice(&99u32.to_be_bytes());
    section[42..46].copy_from_slice(&3u32.to_be_bytes());
    let packed = packed_data(&[], &[(5, &[9])], &[(3, &[0, 2, 7])]);
    // Reference array ends after five bits; data begins at the next octet.
    assert_eq!(packed, vec![0x48, 0x0b, 0x80]);
    assert_eq!(unpack(&section, &packed), vec![118.0, 122.0, 132.0]);
    let bytes = message(&section, &packed, 3, &[]);
    let decoded = read_messages(&bytes).next().unwrap();
    assert_eq!(decoded.grid_dimensions().unwrap(), (1, 3));
    assert!(!decoded.has_bitmap());
    assert_eq!(decoded.data().unwrap(), vec![118.0, 122.0, 132.0]);
}

#[test]
fn implicit_widths_and_lengths_apply_to_multiple_groups_with_distinct_last_length() {
    let mut section = complex_section(2, 0, 7, 3);
    section[19] = 5;
    section[35] = 3;
    section[37..41].copy_from_slice(&2u32.to_be_bytes());
    section[41] = 7; // Ignored when the length-descriptor bit count is zero.
    section[42..46].copy_from_slice(&3u32.to_be_bytes());
    let packed = packed_data(
        &[],
        &[(5, &[1, 3, 7])],
        &[(3, &[0, 7]), (3, &[2, 4]), (3, &[1, 3, 6])],
    );
    assert_eq!(
        unpack(&section, &packed),
        vec![1.0, 8.0, 5.0, 7.0, 8.0, 10.0, 13.0]
    );
}

#[test]
fn implicit_and_encoded_descriptors_are_independent_for_both_templates() {
    for (template, order, spatial, expected) in [
        (2, 0, vec![], vec![0.0, 0.0, 5.0, 7.0, 8.0, 10.0, 13.0]),
        (
            3,
            1,
            vec![10, 0x81],
            vec![10.0, 9.0, 13.0, 19.0, 26.0, 35.0, 47.0],
        ),
        (
            3,
            2,
            vec![10, 13, 0x81],
            vec![10.0, 13.0, 20.0, 33.0, 53.0, 82.0, 123.0],
        ),
    ] {
        // DRT 5.3 has valid one-octet descriptors: first value 10, second
        // value 13 for order two, and sign-magnitude minimum difference -1.
        // Its first `order` packed placeholders are zero.
        for width_bits in [0, 2] {
            for length_bits in [0, 2] {
                let mut section = complex_section(template, order, 7, 3);
                section[19] = 4;
                section[35] = 3;
                section[36] = width_bits;
                section[37..41].copy_from_slice(&2u32.to_be_bytes());
                section[41] = 4;
                section[42..46].copy_from_slice(&3u32.to_be_bytes());
                section[46] = length_bits;
                let mut descriptors: Vec<(u8, &[u32])> = vec![(4, &[0, 3, 7])];
                if width_bits != 0 {
                    descriptors.push((width_bits, &[0, 0, 0]));
                }
                if length_bits != 0 {
                    // Encoded final length would be 14; explicit length 3 wins.
                    descriptors.push((length_bits, &[0, 0, 3]));
                }
                let packed = packed_data(
                    &spatial,
                    &descriptors,
                    &[(3, &[0, 0]), (3, &[2, 4]), (3, &[1, 3, 6])],
                );
                assert_eq!(unpack(&section, &packed), expected,
                    "template {template}, order {order}, width bits {width_bits}, length bits {length_bits}");
            }
        }
    }
}

#[test]
fn spatial_differencing_with_zero_group_reference_bits_uses_implicit_descriptors() {
    for (order, spatial, expected) in [
        (
            1,
            vec![10, 0x81],
            vec![10.0, 9.0, 10.0, 13.0, 13.0, 15.0, 20.0],
        ),
        (
            2,
            vec![10, 13, 0x81],
            vec![10.0, 13.0, 17.0, 24.0, 31.0, 40.0, 54.0],
        ),
    ] {
        let mut section = complex_section(3, order, 7, 3);
        section[35] = 3;
        section[37..41].copy_from_slice(&2u32.to_be_bytes());
        section[42..46].copy_from_slice(&3u32.to_be_bytes());
        let packed = packed_data(
            &spatial,
            &[],
            &[(3, &[0, 0]), (3, &[2, 4]), (3, &[1, 3, 6])],
        );
        assert_eq!(unpack(&section, &packed), expected, "order {order}");
        let bytes = message(&section, &packed, 8, &[0b11101111]);
        let decoded = read_messages(&bytes).next().unwrap();
        let mut mapped = expected;
        mapped.insert(3, f64::NAN);
        assert_values(&decoded.data().unwrap(), &mapped);
    }
}

#[test]
fn wide_byte_straddling_descriptors_are_big_endian_and_final_descriptor_is_ignored() {
    for (template, order, spatial, expected) in [
        (
            2,
            0,
            vec![],
            vec![0.0, 0.0, 3.0, 3.0, 4.0, 5.0, 8.0, 10.0, 8.0, 22.0],
        ),
        (
            3,
            1,
            vec![10, 0x81],
            vec![10.0, 9.0, 11.0, 13.0, 16.0, 20.0, 27.0, 36.0, 43.0, 64.0],
        ),
        (
            3,
            2,
            vec![10, 13, 0x81],
            vec![
                10.0, 13.0, 18.0, 25.0, 35.0, 49.0, 70.0, 100.0, 137.0, 195.0,
            ],
        ),
    ] {
        let mut section = complex_section(template, order, 10, 3);
        section[19] = 5;
        section[35] = 2;
        section[36] = 9;
        section[37..41].copy_from_slice(&1u32.to_be_bytes());
        section[41] = 2;
        section[42..46].copy_from_slice(&2u32.to_be_bytes());
        section[46] = 9;
        // Widths 2,3,4; lengths 3,5,2. Each second/third descriptor crosses
        // an octet. The final encoded length 511 would imply 1023 points.
        let packed = packed_data(
            &spatial,
            &[(5, &[0, 3, 7]), (9, &[0, 1, 2]), (9, &[1, 2, 511])],
            &[(2, &[0, 0, 3]), (3, &[0, 1, 2, 5, 7]), (4, &[1, 15])],
        );
        let start = spatial.len() + 2;
        assert_eq!(
            &packed[start..start + 8],
            &[0, 0, 0x40, 0x40, 0, 0x80, 0xbf, 0xe0]
        );
        assert_eq!(
            unpack(&section, &packed),
            expected,
            "template {template}, order {order}"
        );
    }
}

#[test]
fn empty_decoded_data_maps_to_an_all_missing_bitmap() {
    let bitmap = BitmapSection::from_data(&[0, 0, 0, 8, 6, 0, 0, 0]);
    let values = bitmap.map_data(vec![]);
    assert_eq!(values.len(), 16);
    assert!(values.iter().all(|value| value.is_nan()));
}

#[test]
fn zero_count_all_missing_bitmap_survives_message_decode() {
    // Use ten grid cells to also verify that padding bitmap bits are trimmed.
    // Existing constant_fields tests cover zero-group constants with defined
    // cells; this covers an actually empty decoded stream.
    for (template, order) in [(0, 0), (2, 0), (3, 1), (3, 2), (40, 0), (42, 0)] {
        if template == 40 && !cfg!(feature = "jpeg") {
            continue;
        }
        let mut section = complex_section(template, order, 0, 0);
        if matches!(template, 0 | 40 | 42) {
            let length = match template {
                40 => 23,
                42 => 25,
                _ => 21,
            };
            section.truncate(length);
            section[..4].copy_from_slice(&(length as u32).to_be_bytes());
        }
        section[11..15].copy_from_slice(&545f32.to_be_bytes());
        section[17..19].copy_from_slice(&0x8002u16.to_be_bytes());
        assert!(unpack(&section, &[]).is_empty());
        let bytes = message(&section, &[], 10, &[0, 0]);
        let decoded = read_messages(&bytes).next().unwrap();
        assert!(decoded.has_bitmap());
        assert_eq!(decoded.data_point_count().unwrap(), 0);
        assert_eq!(decoded.grid_dimensions().unwrap(), (1, 10));
        let values = decoded.data().unwrap();
        assert_eq!(values.len(), 10);
        assert!(
            values.iter().all(|value| value.is_nan()),
            "template {template}, order {order}"
        );
    }
}

#[test]
fn nonconstant_simple_packing_remains_a_bitmap_control() {
    let mut section = complex_section(0, 0, 3, 0);
    section.truncate(21);
    section[..4].copy_from_slice(&21u32.to_be_bytes());
    section[11..15].copy_from_slice(&545f32.to_be_bytes());
    section[17..19].copy_from_slice(&0x8002u16.to_be_bytes());
    section[19] = 6;
    let bytes = message(&section, &[0x80, 0, 0], 8, &[0b01001010]);
    let decoded = read_messages(&bytes).next().unwrap();
    assert!(decoded.has_bitmap());
    assert_values(
        &decoded.data().unwrap(),
        &[
            f64::NAN,
            57700.0,
            f64::NAN,
            f64::NAN,
            54500.0,
            f64::NAN,
            54500.0,
            f64::NAN,
        ],
    );
}
