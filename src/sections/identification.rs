extern crate grib_data_derive;

use grib_data_derive::{DisplayDescription, FromValue};
use super::section::{Section, section_length};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum ReferenceDataSignificance {
    Analysis = 0,
    #[description = "start of forecast"]
    StartOfForecast = 1,
    #[description = "verifying time of forecast"]
    VerifyingTimeOfForecast = 2, 
    #[description = "observation time"]
    ObservationTime = 3,
    Missing = 255,
}

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
 
    fn reference_date_significance(&self) -> ReferenceDataSignificance {
        self.data[11].into()
    }
}
