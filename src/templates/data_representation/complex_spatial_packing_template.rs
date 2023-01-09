use std::iter;

use itertools::{izip};

use crate::{
    templates::template::{Template, TemplateType},
    utils::{
        filled_bit_array, from_bits, grib_power, read_f32_from_bytes,
        read_u32_from_bytes, read_u16_from_bytes,
    },
};

use super::{
    tables::{
        GroupSplittingMethod, MissingValueManagement, OriginalFieldValue, SpatialDifferencingOrder,
    },
    DataRepresentationTemplate,
};

pub struct ComplexSpatialPackingDataRepresentationTemplate {
    data: Vec<u8>,
}

impl Template for ComplexSpatialPackingDataRepresentationTemplate {
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
        "grid point data - complex packing with spatial differencing"
    }
}

impl ComplexSpatialPackingDataRepresentationTemplate {
    pub fn new(data: Vec<u8>) -> ComplexSpatialPackingDataRepresentationTemplate {
        ComplexSpatialPackingDataRepresentationTemplate { data }
    }

    pub fn reference_value(&self) -> f32 {
        read_f32_from_bytes(self.data.as_slice(), 11).unwrap_or(0.0)
    }

    pub fn binary_scale_factor(&self) -> i16 {
        as_signed!(read_u16_from_bytes(self.data.as_slice(), 15).unwrap_or(0), i16)
    }

    pub fn decimal_scale_factor(&self) -> i16 {
        as_signed!(read_u16_from_bytes(self.data.as_slice(), 17).unwrap_or(0), i16)
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

    pub fn spatial_differencing_order(&self) -> SpatialDifferencingOrder {
        self.data[47].into()
    }

    pub fn number_of_octets_for_differencing(&self) -> u8 {
        self.data[48]
    }
}

impl DataRepresentationTemplate<f64> for ComplexSpatialPackingDataRepresentationTemplate {
    fn compression_type(&self) -> String {
        "Complex Grid Packing with Spatial Differencing".into()
    }

    fn bit_count_per_datapoint(&self) -> usize {
        self.bit_count() as usize
    }

    fn unpack(&self, bits: Vec<u8>, range: std::ops::Range<usize>) -> Result<Vec<f64>, String> {

        let d1 = unwrap_or_return!(
            from_bits::<u16>(&filled_bit_array::<16>(&bits[0..16])),
            "failed to convert value to u16".into()
        );
        let d1 = as_signed!(d1, i16);

        let d2 = if self.spatial_differencing_order() == SpatialDifferencingOrder::Second {
            unwrap_or_return!(
                from_bits::<u16>(&filled_bit_array::<16>(&bits[16..32])),
                "failed to convert value to u16".into()
            )
        } else {
            0
        };
        let d2 = as_signed!(d2, i16);

        let dmin_start = if self.spatial_differencing_order() == SpatialDifferencingOrder::Second {
            32
        } else {
            16
        };
        let dmin = unwrap_or_return!(
            from_bits::<u16>(&filled_bit_array::<16>(&bits[dmin_start..dmin_start + 16])),
            "failed to convert value to u16".into()
        );
        let dmin = as_signed!(dmin, i16);

        println!("{:?}", &bits[dmin_start..dmin_start + 16]);

        println!("{d1}"); 
        println!("{d2}");
        println!("{dmin}");

        let group_reference_start = self.number_of_octets_for_differencing() as usize * 8 * match self.spatial_differencing_order() {
            SpatialDifferencingOrder::First => 2,
            SpatialDifferencingOrder::Second => 3,
        };

        let ng = self.number_of_groups() as usize;
        let n_reference_bits = self.bit_count() as usize;
        let group_reference_bit_start_index = 32 - n_reference_bits;
        let group_references = (0..ng).map(|ig| {
            let start = group_reference_start + ig * n_reference_bits;
            let mut temp_container: [u8; 32] = [0; 32];
            for i in 0..n_reference_bits {
                temp_container[group_reference_bit_start_index + i] = bits[start + i];
            }

            from_bits::<u32>(&temp_container).unwrap()
        });

        let group_widths_start = group_reference_start + (((ng * n_reference_bits) as f32 / 8.0).ceil() as usize * 8);
        let n_width_bits = self.group_width_bits() as usize;
        let group_width_bit_start_index = 32 - n_width_bits;
        let group_widths = (0..ng).map(|ig| {
            let start = group_widths_start + ig * n_width_bits;
            let mut temp_container: [u8; 32] = [0; 32];
            for i in 0..n_width_bits {
                temp_container[group_width_bit_start_index + i] = bits[start + i];
            }

            from_bits::<u32>(&temp_container).unwrap() + self.group_width_reference() as u32
        });

        let group_lengths_start = group_widths_start + (((ng * n_width_bits) as f32 / 8.0).ceil() as usize * 8);
        let n_length_bits = self.group_length_bits() as usize;
        let group_length_bit_start_index = 32 - n_length_bits;
        let group_lengths = (0..ng - 1).map(|ig| {
            let start = group_lengths_start + ig * n_length_bits;
            let mut temp_container: [u8; 32] = [0; 32];
            for i in 0..n_length_bits {
                temp_container[group_length_bit_start_index + i] = bits[start + i];
            }

            from_bits::<u32>(&temp_container).unwrap() * self.group_length_increment() as u32
                + self.group_length_reference()
        })
        .chain(iter::once(self.group_last_length()));

        let mut pos = group_lengths_start + (((ng * n_length_bits) as f32 / 8.0).ceil() as usize * 8);
        let mut raw_values = Vec::with_capacity(ng);
        for (reference, width, length) in izip!(group_references, group_widths, group_lengths) {
            if width == 0 {
                raw_values.push(vec![0; length as usize]);
                continue;
            }

            let n_bits = (width * length) as usize;
            let mut temp_container: [u8; 32] = [0; 32];
            let bit_start_index = 32 - width as usize;
            let group_values: Vec<i32> = (0..length)
                .map(|i| {
                    temp_container = [0; 32];
                    for bit in 0..width as usize {
                        temp_container[bit_start_index + bit as usize] = bits[pos + (i * width) as usize + bit];
                    }

                    let raw = as_signed!(from_bits::<u32>(&temp_container).unwrap(), i32);
                    //println!("RAW - {raw} == REF - {reference} - DMIN {dmin}");
                    raw + reference as i32
                })
                .collect();

            pos += n_bits;

            raw_values.push(group_values);
        }
        let raw_values: Vec<i32> = raw_values.iter().flatten().map(|v| *v).collect();

        println!("raw_values {:?}", &raw_values[2000..2050]);

        let mut values = Vec::with_capacity(raw_values.len());
        match self.spatial_differencing_order() {
            SpatialDifferencingOrder::First => {
                values.push(d1 as i32);
                for i in 1..raw_values.len() {
                    let val = raw_values[i] + values[i - 1] + dmin as i32;
                    values.push(val);
                }
            },
            SpatialDifferencingOrder::Second => {
                values.push(d1 as i32); 
                values.push(d2 as i32);
                for i in 2..raw_values.len() {
                    let val = raw_values[i] + 2 * values[i - 1] - values[i - 2] + dmin as i32;
                    values.push(val);
                }
            },
        };

        let bscale = 2_f64.powi(self.binary_scale_factor() as i32);
        let dscale = 10_f64.powi(-self.decimal_scale_factor() as i32);
        let reference_value = self.reference_value() as f64;

        println!("bscale {bscale}");
        println!("dscale {dscale}");
        println!("refval {reference_value}");
        // println!("values {:?}", &values);

        // values [55.0, 55.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]

        let values = values
        .iter()
        .map(|v| ((*v as f64) * bscale + reference_value) * dscale).collect::<Vec<f64>>();

        // [5.522778330743313, 5.522778330743313, 0.022778330743312838, 0.022778330743312838, 0.022778330743312838, 0.022778330743312838, 0.022778330743312838, 0.022778330743312838, 0.022778330743312838, 0.022778330743312838]
        println!("values {:?}", &values[2000..2050]);

        Ok(values[range].to_vec())
    }
}