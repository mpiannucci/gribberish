use crate::sections::section::{Section, SectionType};
use crate::field::Field;
use std::vec::Vec;


pub struct Message<'a> {
    pub sections: Vec<Section<'a>>,
    pub fields: Vec<Field<'a>>,
}

impl <'a> Message<'a> {
	pub fn parse(data: &'a[u8], offset: usize) -> Result<Message<'a>, &'static str> {
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

        let fields = Vec::new();
        // TODO: Iterate and unpack all the fields


        Ok(Message {
            sections,
            fields,
        })
	}

    pub fn parse_all(data: &'a[u8]) -> Vec<Message<'a>> {
        let mut messages = Vec::new();
        let mut offset: usize = 0;

        while offset < data.len() {
            if let Ok(message) = Message::parse(data, offset) {
                offset += message.len();
                messages.push(message);
            } else {
                break;
            }
        }

        messages
    }

    pub fn len(&self) -> usize {
        match self.sections.first() {
            Some(section) => match &section.section {
                SectionType::Indicator(indicator) => indicator.total_length() as usize,
                _ => 0,
            },
            None => 0,
        }
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    pub fn field_count(&self) -> usize {
        self.fields.len()
    }
}
