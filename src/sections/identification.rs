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

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum ProductionStatus {
    Operational = 0,
    #[description = "operational test"]
    OperationalTest = 1,
    Research = 2,
    ReAnalysis = 3,
    #[description = "TIGGE"]
    TIGGE = 4,
    #[description = "TIGGE test"]
    TIGGETest = 5,
    #[description = "s2s operational"]
    S2SOperational = 6,
    #[description = "s2s test"]
    S2STest = 7,
    #[description = "UERRA"]
    UERRA = 8,
    #[description = "UERRA test"]
    UERRATest = 9,
    Missing = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum GribDataType {
    Analysis = 0,
    Forecast = 1,
    #[description = "analysis and forecast"]
    AnalysisAndForecast = 2,
    #[description = "control forecast"]
    ControlForecast = 3,
    #[description = "perturbed forecast"]
    PerturbedForecast = 4,
    #[description = "control and perturbed forecast"]
    ControlAndPerturbedForecast = 5, 
    #[description = "processed satellite observations"]
    ProcessedSatelliteObservations = 6,
    #[description = "processed radar observations"]
    ProcessedRadarObservations = 7,
    #[description = "event probability"]
    EventProbability = 8,
    Experimental = 192,
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
    pub fn from_data(data: &[u8], offset: usize) -> IdentificationSection {
        let len = section_length(data, offset);
        IdentificationSection {
            data: &data[offset .. offset+len],
        }
    }
 
    pub fn reference_date_significance(&self) -> ReferenceDataSignificance {
        self.data[11].into()
    }

    pub fn production_status(&self) -> ProductionStatus {
        self.data[19].into()
    }

    pub fn data_type(&self) -> GribDataType {
        self.data[20].into()
    }
}
