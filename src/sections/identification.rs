use super::section::{Section, section_length};


pub struct IdentificationSection<'a>{
    data: &'a[u8],
}

impl Section for IdentificationSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> IdentificationSection<'a> {

    fn from_data(data: &[u8], offset: usize) -> IdentificationSection {
        let len = section_length(data, offset);
        IdentificationSection {
            data: &data[offset .. offset+len],
        }
    }

    fn reference_date_significance_value(&self) -> u8 {
        self.data[11]
    }
 
    fn reference_date_significance(&self) -> &'static str {
        match self.reference_date_significance_value() {
            0 => "analysis",
            1 => "start of forecast",
            2 => "verifying time of forecast",
            3 => "observation time",
            255 => "missing",
            _ => "unknown",
        }
    }

    

}