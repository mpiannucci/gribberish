

pub struct Message<'a> {
	data: &'a[u8],
	offset: usize,
}

impl <'a> Message<'a> {
	pub fn from_data(data: &'a[u8], offset: usize) -> Message<'a> {
		Message {
			data, 
			offset,
		}
	}

	
}