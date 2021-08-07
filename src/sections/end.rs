use std::str;
use super::grib_section::GribSection;

fn validate_end_section(data: &[u8]) -> bool {
    match str::from_utf8(&data[0..4]) {
		Ok(s) => s == "7777",
		_ => false
	}
}

pub struct EndSection {
    data: Vec<u8>,
}

impl EndSection {
    pub fn from_data(data: Vec<u8>) -> EndSection {
        EndSection {
            data: data,
        }
    }

    pub fn is_end_section(data: &[u8], offset: usize) -> bool {
        validate_end_section(&data[offset..offset+4])
    }

    pub fn valid(&self) -> bool {
        validate_end_section(&self.data[0..4])
    }
}

impl GribSection for EndSection {
    fn len(&self) -> usize {
        4
    }

    fn number(&self) -> u8 {
        8
    }
}