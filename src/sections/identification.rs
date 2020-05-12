use super::section::{Section, section_length};
use std::convert::From;
use std::fmt;

#[repr(u8)]
#[derive(Eq, PartialEq, Debug)]
pub enum ReferenceDataSignificance {
    Analysis = 0,
    StartOfForecast = 1,
    VerifyingTimeOfForecast = 2, 
    ObservationTime = 3,
    Missing = 255,
}

impl From<u8> for ReferenceDataSignificance {
    fn from(value: u8) -> Self {
        match value {
            0 => ReferenceDataSignificance::Analysis,
            1 => ReferenceDataSignificance::StartOfForecast,
            2 => ReferenceDataSignificance::VerifyingTimeOfForecast,
            3 => ReferenceDataSignificance::ObservationTime,
            _ => ReferenceDataSignificance::Missing,
        }
    }
}

impl fmt::Display for ReferenceDataSignificance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match self {
            ReferenceDataSignificance::Analysis => "analysis",
            ReferenceDataSignificance::StartOfForecast => "start of forecast",
            ReferenceDataSignificance::VerifyingTimeOfForecast => "verifying time of forecast",
            ReferenceDataSignificance::ObservationTime => "observation time",
            _ => "missing",
        };
        write!(f, "{}", description)
    }
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