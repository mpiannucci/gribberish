use bitvec::prelude::*;
use gribberish::message::read_messages;
use gribberish::sections::data_representation::DataRepresentationSection;

// These single-message fixtures come from the public NOAA RRFS bucket. Both use DRT 5.3
// with zero groups, zero spatial-descriptor octets, missing-value management 0, and no
// bitmap. The grid is 1059 × 1799, and Section 7 has no packed payload.
const NONZERO: &[u8] = include_bytes!("../../test-data/20260915-12-prob-f01-22.grib2");
const ZERO: &[u8] = include_bytes!("../../test-data/20260915-12-sprd-f01-63.grib2");
// HRRR wrfprsf00 DPT at 100 mb from the public NOAA HRRR bucket (hrrr.20261008, t20z).
// DRT 5.0 with a zero bit width, reference value 192.12479 K, no bitmap, and a
// one-octet Section 7. The grid is 1059 × 1799.
const SIMPLE: &[u8] =
    include_bytes!("../../test-data/hrrr.t20z.wrfprsf00-DPT-100mb-constant.grib2");

fn sections(message: &[u8]) -> Vec<Vec<u8>> {
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

fn signed_scale(value: i16) -> [u8; 2] {
    (value.unsigned_abs() | if value < 0 { 0x8000 } else { 0 }).to_be_bytes()
}

fn constant_section(
    template: u16,
    order: u8,
    count: u32,
    binary_scale: i16,
    decimal_scale: i16,
    missing_management: u8,
) -> Vec<u8> {
    let length = match template {
        0 => 21,
        3 => 49,
        _ => 47,
    };
    let mut section = vec![0u8; length];
    section[..4].copy_from_slice(&(length as u32).to_be_bytes());
    section[4] = 5;
    section[5..9].copy_from_slice(&count.to_be_bytes());
    section[9..11].copy_from_slice(&template.to_be_bytes());
    section[11..15].copy_from_slice(&12.5f32.to_be_bytes());
    section[15..17].copy_from_slice(&signed_scale(binary_scale));
    section[17..19].copy_from_slice(&signed_scale(decimal_scale));
    if template == 0 {
        return section;
    }
    section[21] = 1;
    section[22] = missing_management;
    section[23..27].copy_from_slice(&(-9999.0f32).to_be_bytes());
    section[27..31].copy_from_slice(&(-8888.0f32).to_be_bytes());
    section[42..46].copy_from_slice(&99u32.to_be_bytes());
    if template == 3 {
        section[47] = order;
    }
    section
}

fn constant_message(template: u16, order: u8, bitmap: bool) -> Vec<u8> {
    let mut message = NONZERO[..16].to_vec();
    for mut section in sections(NONZERO) {
        match section[4] {
            3 => {
                section[6..10].copy_from_slice(&8u32.to_be_bytes());
                section[30..34].copy_from_slice(&4u32.to_be_bytes());
                section[34..38].copy_from_slice(&2u32.to_be_bytes());
            }
            5 => section = constant_section(template, order, if bitmap { 5 } else { 8 }, 17, 2, 0),
            6 if bitmap => section = vec![0, 0, 0, 7, 6, 0, 0b10110101],
            _ => {}
        }
        message.extend(section);
    }
    message.extend(b"7777");
    let length = message.len() as u64;
    message[8..16].copy_from_slice(&length.to_be_bytes());
    message
}

#[test]
fn zero_groups_use_section5_count_and_unscaled_reference() {
    // GDAL/g2c return 12.5 for zero groups with either sign of decimal scale.
    let empty = BitSlice::<u8, Msb0>::empty();
    for (template, order) in [(2, 0), (3, 1), (3, 2)] {
        for count in [0, 1, 7] {
            for binary_scale in [-17, 0, 17] {
                for decimal_scale in [-2, 0, 2] {
                    for missing_management in [0, 1, 2] {
                        let section = constant_section(
                            template,
                            order,
                            count,
                            binary_scale,
                            decimal_scale,
                            missing_management,
                        );
                        let representation = DataRepresentationSection::from_data(&section);
                        let values = representation
                            .data_representation_template()
                            .unwrap()
                            .unpack(empty)
                            .unwrap();
                        assert_eq!(values, vec![12.5; count as usize]);
                    }
                }
            }
        }
    }
}

#[test]
fn read_real_zero_group_constants() {
    for (fixture, expected) in [(ZERO, 0.0), (NONZERO, 100.00001525878906)] {
        let section5 = sections(fixture)
            .into_iter()
            .find(|section| section[4] == 5)
            .unwrap();
        assert_eq!(&section5[31..35], &[0; 4]);
        assert_eq!(section5[22], 0);
        assert_eq!(section5[48], 0);
        let messages: Vec<_> = read_messages(fixture).collect();
        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(message.grid_dimensions().unwrap(), (1059, 1799));
        assert!(!message.has_bitmap());
        let values = message.data().unwrap();
        assert_eq!(values.len(), 1059 * 1799);
        assert!(values.iter().all(|&value| value == expected));
    }
}

#[test]
fn nonzero_constant_survives_message_decode() {
    for (template, order) in [(0, 0), (2, 0), (3, 1), (3, 2)] {
        let message = constant_message(template, order, false);
        let decoded = read_messages(&message).next().unwrap();
        assert_eq!(decoded.grid_dimensions().unwrap(), (2, 4));
        assert!(!decoded.has_bitmap());
        assert_eq!(decoded.data().unwrap(), vec![12.5; 8]);
    }
}

#[test]
fn zero_group_constants_expand_bitmap_without_losing_missing_cells() {
    for (template, order) in [(0, 0), (2, 0), (3, 1), (3, 2)] {
        let message = constant_message(template, order, true);
        let messages: Vec<_> = read_messages(&message).collect();
        assert_eq!(messages.len(), 1);
        let decoded = &messages[0];
        assert_eq!(decoded.grid_dimensions().unwrap(), (2, 4));
        assert!(decoded.has_bitmap());
        let values = decoded.data().unwrap();
        assert_eq!(values.len(), 8);
        for (index, value) in values.iter().enumerate() {
            if [1, 4, 6].contains(&index) {
                assert!(value.is_nan(), "bitmap cell {index} was {value}");
            } else {
                assert_eq!(*value, 12.5, "defined cell {index}");
            }
        }
    }
}

#[test]
fn zero_bit_simple_packing_uses_section5_count_and_unscaled_reference() {
    // g2c simunpack and ecCodes data_simple_packing return the reference value for
    // every point when the bit width is zero, whatever the scale factors.
    let empty = BitSlice::<u8, Msb0>::empty();
    for count in [0, 1, 7] {
        for binary_scale in [-17, 0, 17] {
            for decimal_scale in [-2, 0, 2] {
                let section = constant_section(0, 0, count, binary_scale, decimal_scale, 0);
                let representation = DataRepresentationSection::from_data(&section);
                let values = representation
                    .data_representation_template()
                    .unwrap()
                    .unpack(empty)
                    .unwrap();
                assert_eq!(values, vec![12.5; count as usize]);
            }
        }
    }
}

#[test]
fn read_real_simple_packing_constant() {
    let section5 = sections(SIMPLE)
        .into_iter()
        .find(|section| section[4] == 5)
        .unwrap();
    assert_eq!(u16::from_be_bytes([section5[9], section5[10]]), 0);
    assert_eq!(section5[19], 0);
    let messages: Vec<_> = read_messages(SIMPLE).collect();
    assert_eq!(messages.len(), 1);
    let message = &messages[0];
    assert_eq!(message.grid_dimensions().unwrap(), (1059, 1799));
    assert!(!message.has_bitmap());
    let values = message.data().unwrap();
    assert_eq!(values.len(), 1059 * 1799);
    assert!(values.iter().all(|&value| value == 192.12478637695312));
}
