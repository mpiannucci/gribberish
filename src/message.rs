use crate::sections::section::{Section, SectionType};
use std::vec::Vec;


pub struct Message<'a> {
	sections: Vec<Section<'a>>,
}

impl <'a> Message<'a> {
	pub fn from_data(data: &'a[u8], offset: usize) -> Result<Message<'a>, &'static str> {
        let mut sections: Vec<Section<'a>> = Vec::new();

        let mut current_offset = 0;
        loop {
            if let Some(section) = sections.last() {
                if let SectionType::End(_) = section.section {
                    break;
                }
            }

            let next_section = Section::from_data(data, offset + current_offset)?;
            if let SectionType::Invalid = next_section.section {
                return Err("Error while reading sections");
            }
            
            current_offset += next_section.len();
            sections.push(next_section);
        }

        Ok(Message {
            sections
        })
	}

    pub fn len(&self) -> usize {
        match self.sections.first() {
            Some(section) => section.len(),
            None => 0,
        }
    }
}
