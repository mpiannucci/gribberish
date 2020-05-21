use grib_macros::{DisplayDescription, FromValue};
use super::template::{Template, TemplateType};
use crate::utils::{read_f32_from_bytes, read_i16_from_bytes, bits_from_bytes};

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

pub enum DataRepresentationTemplate {
	SimpleGridPoint,
	Other,
}

pub struct SimpleGridPointDataTemplate<'a> {
	data: &'a[u8],
	discipline: u8,
}

impl <'a> Template for SimpleGridPointDataTemplate<'a> {
	fn data(&self) -> &[u8] {
    	self.data
 	}

 	fn template_number(&self) -> u16 {
 	    0
 	}

 	fn template_type(&self) -> TemplateType {
 	    TemplateType::DataRepresentation
 	}
}

impl <'a> SimpleGridPointDataTemplate<'a> {
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