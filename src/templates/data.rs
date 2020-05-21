use super::template::{Template, TemplateType};
use crate::utils::bit_array_from_bytes;

pub enum DataTemplate<'a> {
	SimpleGridPoint(SimpleGridPointDataTemplate<'a>),
	Other,
}

impl<'a> DataTemplate<'a> {
	pub fn from_template_number(template_number: u16, data: &'a[u8], bit_size: u8) -> DataTemplate {
		match template_number {
			0 => DataTemplate::SimpleGridPoint(SimpleGridPointDataTemplate{data, bit_size}),
			_ => DataTemplate::Other,
		}
	}
}

pub struct SimpleGridPointDataTemplate<'a> {
	data: &'a[u8],
	bit_size: u8,
}

impl <'a> Template for SimpleGridPointDataTemplate<'a> {
	fn data(&self) -> &[u8] {
    	self.data
 	}

 	fn template_number(&self) -> u16 {
 	    0
 	}

 	fn template_type(&self) -> TemplateType {
 	    TemplateType::Data
 	}

 	fn template_name(&self) -> &str {
 		"grid point data - simple packing"
 	}
}

impl <'a> SimpleGridPointDataTemplate<'a> {
	pub fn bits_per_datapoint(&self) -> u8 {
		self.bit_size
	}

	pub fn bytes_per_datapoint(&self) -> u8 {
		self.bit_size / 8
	}

	pub fn data_point_count(&self) -> usize {
		if self.bit_size < 1 {
			return 0
		}

		((self.data.len() - 5) * 8) / self.bit_size as usize
	}

	pub fn raw_data_bytes(&self) -> &[u8] {
		&self.data[5..]
	}

	pub fn raw_bit_data(&self) -> Vec<u8> {
		bit_array_from_bytes(&self.data[5..])
	}
}