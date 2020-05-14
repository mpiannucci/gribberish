use super::section::{Section, section_length};

pub struct LocalUseSection<'a> {
    data: &'a[u8],
}

impl Section for LocalUseSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> LocalUseSection<'a> {
    pub fn from_data(data: &[u8], offset: usize) -> LocalUseSection {
        let len = section_length(data, offset);
        LocalUseSection {
            data: &data[offset .. offset+len],
        }
    }

    pub fn exists(&self) -> bool {
        self.number() == 2
    }
}