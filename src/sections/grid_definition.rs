extern crate grib_data_derive;

use crate::sections::section::{Section, section_length};
use crate::utils::{read_u16_from_bytes, read_u32_from_bytes};
use grib_data_derive::{DisplayDescription, FromValue};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum GridSource {
    #[description = "specified in code"]
    Code = 0,
    Predetermined = 1,
    #[description = "not applicable"]
    NotApplicable = 255,
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum NumberListInterpretation {
    #[description = "no appended list"]
    None = 0, 
    #[description = "Numbers define number of points corresponding to full coordinate circles (i.e. parallels).  Coordinate values on each circle are multiple of the circle mesh, and extreme coordinate values given in grid definition may not be reached in all rows."]
    Parallels = 1,
    #[description = "Numbers define number of points corresponding to coordinate lines delimited by extreme coordinate values given in grid definition which are present in each row."]
    CoordinateLines = 2,
    #[description = "Numbers define the actual latitudes for each row in the grid. The list of numbers are integer values of the valid latitudes in microdegrees (scale by 106) or in unit equal to the ratio of the basic angle and the subdivisions number for each row, in the same order as specified in the 'scanning mode flag' (bit no. 2)"]
    ActualLatitudes = 3, 
    Missing = 255,
}

pub struct GridDefinitionSection<'a>{
    data: &'a[u8],
}

impl Section for GridDefinitionSection<'_> {
    fn data(&self) -> &[u8] {
        self.data
    }
}

impl<'a> GridDefinitionSection<'a> {
    fn from_data(data: &[u8], offset: usize) -> GridDefinitionSection {
        let len = section_length(data, offset);
        GridDefinitionSection {
            data: &data[offset .. offset+len],
        }
    }

    fn grid_source(&self) -> GridSource {
        self.data[5].into()
    }

    fn data_point_count(&self) -> usize {
        read_u32_from_bytes(self.data, 6).unwrap_or(0) as usize
    }

    fn optional_defining_number(&self) -> u8 {
        self.data[10]
    }

    fn defining_number_interpretation(&self) -> NumberListInterpretation {
        self.data[11].into()
    }

    fn grid_definition_template_number(&self) -> u16 {
        read_u16_from_bytes(self.data, 12).unwrap_or(0)
    }
}