use grib_macros::{DisplayDescription, FromValue};
use super::template::{Template, TemplateType};
use crate::utils::{read_f32_from_bytes, read_i16_from_bytes, from_bits};
use crate::unwrap_or_return;
use num::{Float};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum OriginalFieldValue {
	FloatingPoint = 0,
	Integer = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum MatrixCoordinateValueFunctions {
	ExplicitCoordinateValueSet = 0,
	LinearCoordinates = 1,
	GeometricCoordinates = 11,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum MatrixCoordinateParameters {
	DirectionDegreesTrue = 1, 
	Frequency = 2,
	RadialNumber = 3,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum GroupSplittingMethods {
	RowByRow = 0, 
	GeneralGroup = 1,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum MissingValueManagement {
	#[description = "no explicit missing values included with the data values"]
	NoMissingValues = 0,
	#[description = "primary missing values included within the data value"]
	IncludesMissingPrimary = 1, 
	#[description = "primary and secondary missing values included within the data values"]
	IncludesMissingPrimarySecondary = 2,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum SpatialDifferencingOrder {
	#[description = "first order spatial differencing"]
	First = 1,
	#[description = "second order spatial differencing"] 
	Second = 2
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum FloatingPointPrecision {
	#[description = "IEEE 32 bit"]
	IEEE32Bit = 1,
	#[description = "IEEE 64 bit"]
	IEEE64Bit = 2,
	#[description = "IEEE 128 bit"]
	IEEE128Bit = 3,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum CompressionType {
	Lossless = 0,
	Lossy = 1,
}

pub trait DataRepresentationTemplate<T> {
	fn scaled_value(&self, raw_value: T) -> T;
	fn unpack(&self, bits: Vec<u8>) -> Result<Vec<T>, &'static str>;
}

pub struct SimpleGridPointDataRepresentationTemplate<'a> {
	data: &'a[u8],
}

impl <'a> Template for SimpleGridPointDataRepresentationTemplate<'a> {
	fn data(&self) -> &[u8] {
    	self.data
 	}

 	fn template_number(&self) -> u16 {
 	    0
 	}

 	fn template_type(&self) -> TemplateType {
 	    TemplateType::DataRepresentation
 	}

 	fn template_name(&self) -> &str {
 		"grid point data - simple packing"
 	}
}

impl <'a> SimpleGridPointDataRepresentationTemplate<'a> {
	pub fn new(data: &'a[u8]) -> SimpleGridPointDataRepresentationTemplate {
		SimpleGridPointDataRepresentationTemplate{data}
	}

	pub fn reference_value(&self) -> f32 {
		read_f32_from_bytes(self.data, 11).unwrap_or(0.0)
	}

	pub fn binary_scale_factor(&self) -> i16 {
		read_i16_from_bytes(self.data, 15).unwrap_or(0)
	}

	pub fn decimal_scale_factor(&self) -> i16 {
		read_i16_from_bytes(self.data, 17).unwrap_or(0)
	}

	pub fn bit_count(&self) -> u8 {
		self.data[19]
	}

	pub fn original_field_value(&self) -> OriginalFieldValue {
		self.data[20].into()
	}
}

impl <'a> DataRepresentationTemplate<f64> for SimpleGridPointDataRepresentationTemplate<'a> {

	fn scaled_value(&self, raw_value: f64) -> f64 {
		let reference_value: f64 = self.reference_value().into();
        let binary_scale_factor: i32 = self.binary_scale_factor().into();
		let decimal_scale_factor: i32 = self.decimal_scale_factor().into();
		
		(reference_value + (raw_value * 2.0.powi(binary_scale_factor))) / 10.0.powi(decimal_scale_factor)
	}

    fn unpack(&self, bits: Vec<u8>) -> Result<Vec<f64>, &'static str> {
        let mut v = Vec::new();
            
        let bits_per_val: usize = self.bit_count().into();
        let bit_start_index: usize = 64 - bits_per_val;

        let mut raw_value: f64 = 0.0;
        let mut val_bits: [u8; 32] = [0; 32];

        for i in (0..bits.len()).step_by(bits_per_val) {
        	val_bits = [0; 32];

            let relevent_bits = &bits[i..i+bits_per_val];
            for (j, bit) in relevent_bits.iter().enumerate() {
            	val_bits[j + bit_start_index] = *bit;
			}
		
            raw_value = unwrap_or_return!(from_bits::<u32>(&val_bits), "failed to convert value to u32").into();
            let val = self.scaled_value(raw_value);
            v.push(val);
        }

        Ok(v)
    }
}