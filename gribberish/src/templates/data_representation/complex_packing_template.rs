use crate::sections::data_representation::DataRepresentationSection;
use bitvec::prelude::*;

use crate::{error::GribberishError, utils::iter::ScaleGribValueIterator};
use itertools::izip;

use crate::{
    templates::template::{Template, TemplateType},
    utils::{read_f32_from_bytes, read_u16_from_bytes, read_u32_from_bytes},
};

use super::{
    tables::{GroupSplittingMethod, MissingValueManagement, OriginalFieldValue},
    DataRepresentationTemplate,
};

pub struct ComplexPackingDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for ComplexPackingDataRepresentationTemplate {
    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn template_number(&self) -> u16 {
        2
    }

    fn template_type(&self) -> TemplateType {
        TemplateType::DataRepresentation
    }

    fn template_name(&self) -> &str {
        "grid point data - complex packing"
    }
}

impl ComplexPackingDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> ComplexPackingDataRepresentationTemplate {
        ComplexPackingDataRepresentationTemplate { data }
    }

    pub fn reference_value(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 11).unwrap_or(0.0)
    }

    pub fn binary_scale_factor(&self) -> i16 {
        as_signed!(
            read_u16_from_bytes(self.data.as_slice(), 15).unwrap_or(0),
            16,
            i16
        )
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        as_signed!(
            read_u16_from_bytes(self.data.as_slice(), 17).unwrap_or(0),
            16,
            i16
        )
    }

    pub fn bit_count(&self) -> u8 {
        self.data[19]
    }

    pub fn original_field_value(&self) -> OriginalFieldValue {
        self.data[20].into()
    }

    pub fn group_splitting_method(&self) -> GroupSplittingMethod {
        self.data[21].into()
    }

    pub fn missing_value_management(&self) -> MissingValueManagement {
        self.data[22].into()
    }

    pub fn primary_missing_value_substitute(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 23).unwrap_or(0.0)
    }

    pub fn secondary_missing_value_substitute(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 27).unwrap_or(0.0)
    }

    pub fn number_of_groups(&self) -> u32 {
        read_u32_from_bytes(self.data.as_slice(), 31).unwrap()
    }

    pub fn group_width_reference(&self) -> u8 {
        self.data[35]
    }

    pub fn group_width_bits(&self) -> u8 {
        self.data[36]
    }

    pub fn group_length_reference(&self) -> u32 {
        read_u32_from_bytes(self.data.as_slice(), 37).unwrap()
    }

    pub fn group_length_increment(&self) -> u8 {
        self.data[41]
    }

    pub fn group_last_length(&self) -> u32 {
        read_u32_from_bytes(self.data.as_slice(), 42).unwrap()
    }

    pub fn group_length_bits(&self) -> u8 {
        self.data[46]
    }
}

impl DataRepresentationTemplate<f64> for ComplexPackingDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "Complex Grid Packing".into()
    }

    fn bit_count_per_datapoint(&self) -> usize {
        self.bit_count() as usize
    }

    fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<f64>, GribberishError> {
        let ng = self.number_of_groups() as usize;
        if ng == 0 {
            let count = DataRepresentationSection::from_data(&self.data).data_point_count();
            // Decode zero-group constants as the reference value without scaling.
            // This follows a long-standing NCEP convention arising from a decimal-scaling bug:
            // https://www.cpc.ncep.noaa.gov/products/wesley/wgrib2/g2clib.html
            // We match wgrib2, ecCodes and GDAL default behavior.
            return Ok(vec![self.reference_value() as f64; count]);
        }
        let nbits = self.bit_count() as usize;

        let group_references = (0..ng).map(|ig| {
            if nbits == 0 {
                0
            } else {
                let start = ig * nbits;
                bits[start..start + nbits].load_be::<u32>()
            }
        });

        let group_widths_start = ((ng * nbits) as f32 / 8.0).ceil() as usize * 8;
        let n_width_bits = self.group_width_bits() as usize;
        let group_widths = (0..ng).map(|ig| {
            if n_width_bits == 0 {
                self.group_width_reference() as u32
            } else {
                let start = group_widths_start + ig * n_width_bits;
                bits[start..start + n_width_bits].load_be::<u32>()
                    + self.group_width_reference() as u32
            }
        });

        let group_lengths_start =
            group_widths_start + (((n_width_bits * ng) as f32 / 8.0).ceil() as usize * 8);
        let n_length_bits = self.group_length_bits() as usize;
        let group_lengths = (0..ng).map(|ig| {
            // The last group uses its explicit length, not the scaled descriptor.
            if ig == ng - 1 {
                self.group_last_length()
            } else if n_length_bits == 0 {
                self.group_length_reference()
            } else {
                let start = group_lengths_start + ig * n_length_bits;
                bits[start..start + n_length_bits].load_be::<u32>()
                    * self.group_length_increment() as u32
                    + self.group_length_reference()
            }
        });

        let mut pos =
            group_lengths_start + (((n_length_bits * ng) as f32 / 8.0).ceil() as usize * 8);

        // GRIB2 template 5.2 note 10 puts zero-width missing patterns in the group reference.
        let missing_management = self.missing_value_management();
        let has_primary_missing = missing_management != MissingValueManagement::NoMissingValues;
        let has_secondary_missing =
            missing_management == MissingValueManagement::IncludesMissingPrimarySecondary;
        let is_missing = move |value: u32, field_bits: usize| -> bool {
            let all_ones = if field_bits == 0 {
                0
            } else {
                u32::MAX >> (32 - field_bits)
            };
            (has_primary_missing && value == all_ones)
                || (has_secondary_missing && field_bits != 0 && value == all_ones - 1)
        };

        let values = izip!(group_references, group_widths, group_lengths)
            .flat_map(|(reference, width, length)| {
                let n_bits = (width * length) as usize;
                let group_values = (0..length).map(move |i| {
                    let value = if width == 0 {
                        0u32
                    } else {
                        bits[pos + (i * width) as usize
                            ..pos + (i * width) as usize + width as usize]
                            .load_be::<u32>()
                    };
                    let missing = if width == 0 {
                        is_missing(reference, nbits)
                    } else {
                        is_missing(value, width as usize)
                    };
                    if missing {
                        f64::NAN
                    } else {
                        let raw = as_signed!(value, 32, i32);
                        (raw + reference as i32) as f64
                    }
                });

                pos += n_bits;

                group_values
            })
            .scale_value_by(
                self.binary_scale_factor(),
                self.decimal_scale_factor(),
                self.reference_value(),
            )
            .collect();

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Section 5 (47 bytes, DRT 5.2) and the section 7 payload: group references, widths,
    // lengths, then packed values.
    const PRIMARY_SECTION5: [u8; 47] = [
        0x00, 0x00, 0x00, 0x2f, 0x05, 0x00, 0x00, 0x00, 0x09, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x01, 0x62, 0x58, 0xd1, 0x9a, 0xff, 0xff, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x03, 0x04,
    ];
    const PRIMARY_PACKED: [u8; 9] = [0x0a, 0xff, 0x05, 0x30, 0x00, 0x42, 0x30, 0x07, 0xb0];
    const SECONDARY_SECTION5: [u8; 47] = [
        0x00, 0x00, 0x00, 0x2f, 0x05, 0x00, 0x00, 0x00, 0x09, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x02, 0x62, 0x58, 0xd1, 0x9a, 0xff, 0xff, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x03, 0x04,
    ];
    const SECONDARY_PACKED: [u8; 9] = [0x0a, 0xfe, 0x05, 0x30, 0x00, 0x42, 0x30, 0x1b, 0xb0];
    const NONE_SECTION5: [u8; 47] = [
        0x00, 0x00, 0x00, 0x2f, 0x05, 0x00, 0x00, 0x00, 0x09, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x00, 0x62, 0x58, 0xd1, 0x9a, 0xff, 0xff, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x03, 0x04,
    ];
    const NONE_PACKED: [u8; 9] = [0x0a, 0xff, 0x05, 0x30, 0x00, 0x42, 0x30, 0x07, 0xb0];
    const NB32_SECTION5: [u8; 47] = [
        0x00, 0x00, 0x00, 0x2f, 0x05, 0x00, 0x00, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x20, 0x00, 0x01, 0x01, 0x62, 0x58, 0xd1, 0x9a, 0xff, 0xff, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x02, 0x04,
    ];
    const NB32_PACKED: [u8; 6] = [0xff, 0xff, 0xff, 0xff, 0x00, 0x20];

    fn unpack_values(section5: &[u8], packed: &[u8]) -> Vec<f64> {
        ComplexPackingDataRepresentationTemplate::new(section5.to_vec())
            .unpack(packed.view_bits::<Msb0>())
            .unwrap()
    }

    fn assert_values(actual: Vec<f64>, expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!(
                (a.is_nan() && e.is_nan()) || a == e,
                "actual {actual:?} expected {expected:?}"
            );
        }
    }

    #[test]
    fn primary_missing_in_packed_values_and_in_width_zero_group_reference() {
        let nan = f64::NAN;
        assert_values(
            unpack_values(&PRIMARY_SECTION5, &PRIMARY_PACKED),
            &[10.0, 11.0, nan, 13.0, nan, nan, 5.0, 5.0, 5.0],
        );
    }

    #[test]
    fn secondary_pattern_is_missing_only_with_secondary_management() {
        let nan = f64::NAN;
        assert_values(
            unpack_values(&SECONDARY_SECTION5, &SECONDARY_PACKED),
            &[10.0, nan, nan, 13.0, nan, nan, 5.0, 5.0, 5.0],
        );
        let mut section5 = SECONDARY_SECTION5;
        section5[22] = 1;
        assert_values(
            unpack_values(&section5, &SECONDARY_PACKED),
            &[10.0, 16.0, nan, 13.0, 254.0, 254.0, 5.0, 5.0, 5.0],
        );
    }

    #[test]
    fn all_ones_is_a_number_without_missing_value_management() {
        assert_values(
            unpack_values(&NONE_SECTION5, &NONE_PACKED),
            &[10.0, 11.0, 17.0, 13.0, 255.0, 255.0, 5.0, 5.0, 5.0],
        );
    }

    #[test]
    fn missing_reference_at_32_bits_does_not_overflow() {
        assert_values(
            unpack_values(&NB32_SECTION5, &NB32_PACKED),
            &[f64::NAN, f64::NAN],
        );
    }

    #[test]
    fn zero_bit_group_reference_respects_missing_management() {
        let mut section5 = NB32_SECTION5;
        section5[19] = 0;
        assert_values(unpack_values(&section5, &[0, 0x20]), &[f64::NAN, f64::NAN]);
        section5[22] = 0;
        assert_values(unpack_values(&section5, &[0, 0x20]), &[0.0, 0.0]);
    }

    #[test]
    fn defined_16_bit_reference_is_not_a_secondary_missing_marker() {
        for management in [2, 1, 0] {
            let mut section5 = NB32_SECTION5;
            section5[19] = 16;
            section5[22] = management;
            assert_values(
                unpack_values(&section5, &[0xfe, 0xff, 0x00, 0x20]),
                &[65279.0, 65279.0],
            );
        }
    }

    #[test]
    fn secondary_16_bit_reference_respects_missing_management() {
        for management in [2, 1, 0] {
            let mut section5 = NB32_SECTION5;
            section5[19] = 16;
            section5[22] = management;
            let expected = if management == 2 { f64::NAN } else { 65534.0 };
            assert_values(
                unpack_values(&section5, &[0xff, 0xfe, 0x00, 0x20]),
                &[expected, expected],
            );
        }
    }

    #[test]
    fn primary_16_bit_reference_respects_missing_management() {
        for management in [2, 1, 0] {
            let mut section5 = NB32_SECTION5;
            section5[19] = 16;
            section5[22] = management;
            let expected = if management == 0 { 65535.0 } else { f64::NAN };
            assert_values(
                unpack_values(&section5, &[0xff, 0xff, 0x00, 0x20]),
                &[expected, expected],
            );
        }
    }

    #[test]
    fn byte_straddling_reference_is_decoded_before_missing_classification() {
        let mut section5 = NB32_SECTION5;
        section5[19] = 7;
        section5[31..35].copy_from_slice(&2u32.to_be_bytes());
        section5[36] = 8;
        section5[42..46].copy_from_slice(&1u32.to_be_bytes());
        section5[46] = 8;
        for management in [2, 1, 0] {
            section5[22] = management;
            for reference in [125u8, 126, 127] {
                let missing =
                    (management != 0 && reference == 127) || (management == 2 && reference == 126);
                let expected = if missing { f64::NAN } else { reference as f64 };
                assert_values(
                    unpack_values(&section5, &[0x0b, reference << 2, 0, 0, 1, 1]),
                    &[5.0, expected],
                );
            }
        }
    }

    #[test]
    fn secondary_missing_reference_at_32_bits_does_not_overflow() {
        let mut section5 = NB32_SECTION5;
        section5[22] = 2;
        assert_values(
            unpack_values(&section5, &[0xff, 0xff, 0xff, 0xfe, 0x00, 0x20]),
            &[f64::NAN, f64::NAN],
        );
    }

    #[test]
    fn packed_32_bit_missing_markers_do_not_overflow() {
        let mut section5 = NB32_SECTION5;
        section5[5..9].copy_from_slice(&3u32.to_be_bytes());
        section5[19] = 8;
        section5[36] = 8;
        section5[42..46].copy_from_slice(&3u32.to_be_bytes());
        section5[46] = 8;
        section5[22] = 2;
        assert_values(
            unpack_values(
                &section5,
                &[
                    0, 32, 3, 0, 0, 0, 7, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
                ],
            ),
            &[7.0, f64::NAN, f64::NAN],
        );
        section5[22] = 1;
        assert_values(
            unpack_values(
                &section5,
                &[0, 32, 3, 0, 0, 0, 7, 0xff, 0xff, 0xff, 0xff, 0, 0, 0, 8],
            ),
            &[7.0, f64::NAN, 8.0],
        );
    }
}
