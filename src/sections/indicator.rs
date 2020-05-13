extern crate grib_data_derive;

use std::str;
use std::convert::From;
use std::fmt;
use grib_data_derive::DisplayDescription;
use super::section::Section;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription)]
pub enum Discipline {
	Meteorological = 0,
	Hydrological = 1,
	LandSurface = 2,
	Space = 3,
	Oceanographic = 10,
	Missing = 255,
}

impl From<u8> for Discipline {
	fn from(value: u8) -> Self {
		match value {
			0 => Discipline::Meteorological,
			1 => Discipline::Hydrological,
			2 => Discipline::LandSurface,
			3 => Discipline::Space,
			10 => Discipline::Oceanographic,
			_ => Discipline::Missing,
		}
	}
}

pub struct IndicatorSection<'a>{
    data: &'a[u8],
}

impl Section for IndicatorSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> IndicatorSection<'a> {

	pub fn from_data(data: &[u8], offset: usize) -> IndicatorSection {
		IndicatorSection {
            data: &data[offset .. offset + 16],
		}
	}

	pub fn valid(&self) -> bool {
		match str::from_utf8(&self.data[0..4]) {
			Ok(s) => s == "GRIB",
			_ => false
		}
	}

	pub fn discipline(&self) -> Discipline {
		self.data[6].into()
	}

	pub fn edition(&self) -> u8 {
		self.data[7]
	}

	pub fn total_length(&self) -> u64{
		let mut l: [u8; 8] = std::default::Default::default();
		l.copy_from_slice(&self.data[8..]); 
		u64::from_be_bytes(l)
	}
}

mod tests {
	use super::IndicatorSection;
	use super::Discipline;
    use super::Section;

    #[test]
    fn read_indicator() {
		let raw: [u8; 16] = [0x47, 0x52, 0x49, 0x42, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb3];
		let indicator = IndicatorSection::from_data(&raw, 0);
        assert!(indicator.valid());
		assert!(indicator.number() == 0);
		assert!(indicator.discipline() == Discipline::Meteorological);
	}
}