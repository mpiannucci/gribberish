use crate::utils::read_u32_from_bytes;

pub fn section_length(data: &[u8], offset: usize) -> usize {
    read_u32_from_bytes(data, offset).unwrap_or(0) as usize
}

pub trait Section {
    fn data(&self) -> &[u8];

    fn length(&self) -> usize {
        read_u32_from_bytes(self.data(), 0).unwrap_or(0) as usize
    }

    fn number(&self) -> u8 {
        self.data()[4]
    }
}