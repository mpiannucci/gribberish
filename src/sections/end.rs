use std::str;

pub struct EndSection<'a> {
    data: &'a[u8],
}

impl<'a> EndSection<'a> {
    pub fn from_data(data: &[u8]) -> EndSection {
        EndSection {
            data: &data,
        }
    }

    pub fn valid(&self) -> bool {
        match str::from_utf8(&self.data[0..4]) {
			Ok(s) => s == "7777",
			_ => false
		}
    }
}