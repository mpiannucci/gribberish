use std::str;
use super::section::{Section, section_length};

struct EndSection<'a> {
    data: &'a[u8],
}

impl Section for EndSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> EndSection<'a> {
    fn from_data(data: &[u8], offset: usize) -> EndSection {
        EndSection {
            data: &data[offset .. offset+4],
        }
    }

    fn valid(&self) -> bool {
        match str::from_utf8(&self.data[0..4]) {
			Ok(s) => s == "7777",
			_ => false
		}
    }
}