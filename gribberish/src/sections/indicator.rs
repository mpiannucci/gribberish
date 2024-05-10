use std::str;
use gribberish_macros::{DisplayDescription, FromValue};
use crate::utils::read_u64_from_bytes;
use super::grib_section::GribSection;

fn validate_indicator_section(data: &[u8]) -> bool {
	match str::from_utf8(&data[0..4]) {
		Ok(s) => s == "GRIB",
		_ => false
	}
}

#[repr(u8)]
#[derive(Eq, Clone, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum Discipline {
	Meteorological = 0,
	Hydrological = 1,
	LandSurface = 2,
	Space = 3,
	Oceanographic = 10,
	MultiRadarMultiSensor = 209,
	Missing = 255,
}

pub struct IndicatorSection<'a> {
    data: &'a [u8],
}

impl <'a> IndicatorSection<'a> {

	pub fn from_data(data: &'a [u8]) -> Self {
		IndicatorSection {
            data,
		}
	}

    pub fn is_indicator_section(data: &[u8], offset: usize) -> bool {
        validate_indicator_section(&data[offset..offset+4])
    }

	pub fn valid(&self) -> bool {
        validate_indicator_section(&self.data[0..4])
    }

	pub fn discipline_value(&self) -> u8 {
		self.data[6]
	}

	pub fn discipline(&self) -> Discipline {
		self.data[6].into()
	}

	pub fn edition(&self) -> u8 {
		self.data[7]
	}

	pub fn total_length(&self) -> u64 {
		read_u64_from_bytes(self.data, 8).unwrap_or(0) as u64
	}
}

impl <'a> GribSection for IndicatorSection<'a> {
    fn len(&self) -> usize {
        16
    }

    fn number(&self) -> u8 {
        0
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_indicator() {
		use super::{IndicatorSection, Discipline};

		let raw = [0x47u8, 0x52, 0x49, 0x42, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xb3];
		let indicator = IndicatorSection::from_data(&raw);
        assert!(indicator.valid());
		assert!(indicator.discipline() == Discipline::Meteorological);
	}
}
