use std::str;
use grib_macros::{DisplayDescription, FromValue};
use crate::utils::read_u64_from_bytes;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum Discipline {
	Meteorological = 0,
	Hydrological = 1,
	LandSurface = 2,
	Space = 3,
	Oceanographic = 10,
	Missing = 255,
}

pub struct IndicatorSection<'a>{
    data: &'a[u8],
}

impl<'a> IndicatorSection<'a> {

	pub fn from_data(data: &[u8]) -> IndicatorSection {
		IndicatorSection {
            data: &data,
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
		read_u64_from_bytes(self.data, 8).unwrap_or(0) as u64
	}
}

mod tests {
	use super::IndicatorSection;
	use super::Discipline;

    #[test]
    fn read_indicator() {
		let raw: [u8; 16] = [0x47, 0x52, 0x49, 0x42, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb3];
		let indicator = IndicatorSection::from_data(&raw);
        assert!(indicator.valid());
		assert!(indicator.discipline() == Discipline::Meteorological);
	}
}
