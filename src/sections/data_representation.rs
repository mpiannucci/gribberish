use super::section::{Section, section_length};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};

struct DataRepresentationSection<'a> {
    data: &'a[u8],
}

impl Section for DataRepresentationSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> DataRepresentationSection<'a> {
    fn from_data(data: &[u8], offset: usize) -> DataRepresentationSection {
        let len = section_length(data, offset);
        DataRepresentationSection {
            data: &data[offset .. offset+len],
        }
    }
    
    fn data_point_count(&self) -> usize {
        read_u32_from_bytes(self.data, 5).unwrap_or(0) as usize
    }

    fn data_representation_template_number(&self) -> u16 {
        read_u16_from_bytes(self.data, 9).unwrap_or(0)
    }
}
