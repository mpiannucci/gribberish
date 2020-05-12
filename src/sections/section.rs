use std::default::Default;

pub fn section_length(data: &[u8], offset: usize) -> usize {
    let mut l: [u8; 4] = Default::default();
    l.copy_from_slice(&data[offset..offset + 4]);
    u32::from_be_bytes(l) as usize
}

pub trait Section {
    fn data(&self) -> &[u8];

    fn length(&self) -> usize {
        let mut l: [u8; 4] = Default::default();
        l.copy_from_slice(&self.data()[0..4]);
        u32::from_be_bytes(l) as usize
    }

    fn number(&self) -> u8 {
        self.data()[4]
    }
}