use grib_macros::{DisplayDescription, FromValue};
use super::template::{Template, TemplateType};
use crate::utils::{read_u32_from_bytes, bit_array_from_bytes};
use std::vec::Vec;

pub enum GridDefinitionTemplate<'a> {
    LatitudeLongitude(LatitudeLongitudeGridTemplate<'a>),
    RotatedLatitudeLongitude,
    StretchedLatitudeLongitude,
    StetchedAndRotatedLatitudeLongitude,
    Mercator, 
    PolarStereographic,
    LambertConformal, 
    GaussianLatitudeLongitude,
    RotatedGaussianLatitudeLongitude,
    StretchedGaussianLatitudeLongitude,
    StretchedAndRotatedGaussianLatitudeLongitude,
    SphericalHarmonicCoefficients,
    RotatedSphericalHarmonicCoefficients,
    StretchedSphericalHarmonicCoefficients,
    StretchedAndRotatedSphericalHarmonicCoefficients,
    SpaceViewPerspectiveOrthographic,
    TriangularGrid,
    EquitorialAzimuthalEquidistantProjection,
    AzimuthRangeProjection,
    CrossSectionGrid,
    HovmollerDiagramGrid, 
    TimeSectionGrid,
    Missing,
}

impl<'a> GridDefinitionTemplate<'a> {
    pub fn from_template_number(template_number: u16, data: &'a[u8]) -> Self {
        match template_number {
            0 => GridDefinitionTemplate::LatitudeLongitude(LatitudeLongitudeGridTemplate{data}),
            _ => GridDefinitionTemplate::Missing,
        }
    }

    pub fn template_number(&self) -> u16 {
        match self {
            GridDefinitionTemplate::LatitudeLongitude(_) => 0,
            GridDefinitionTemplate::RotatedLatitudeLongitude => 1,
            GridDefinitionTemplate::StretchedLatitudeLongitude => 2,
            GridDefinitionTemplate::StetchedAndRotatedLatitudeLongitude => 3,
            GridDefinitionTemplate::Mercator => 10,
            GridDefinitionTemplate::PolarStereographic => 20, 
            GridDefinitionTemplate::LambertConformal => 30, 
            GridDefinitionTemplate::GaussianLatitudeLongitude => 40, 
            GridDefinitionTemplate::RotatedGaussianLatitudeLongitude => 41, 
            GridDefinitionTemplate::StretchedGaussianLatitudeLongitude => 42, 
            GridDefinitionTemplate::StretchedAndRotatedGaussianLatitudeLongitude => 43,
            GridDefinitionTemplate::SphericalHarmonicCoefficients => 50, 
            GridDefinitionTemplate::RotatedSphericalHarmonicCoefficients => 51, 
            GridDefinitionTemplate::StretchedSphericalHarmonicCoefficients => 52, 
            GridDefinitionTemplate::StretchedAndRotatedSphericalHarmonicCoefficients => 53, 
            GridDefinitionTemplate::SpaceViewPerspectiveOrthographic => 90, 
            GridDefinitionTemplate::TriangularGrid => 100, 
            GridDefinitionTemplate::EquitorialAzimuthalEquidistantProjection => 110,
            GridDefinitionTemplate::AzimuthRangeProjection => 120, 
            GridDefinitionTemplate::CrossSectionGrid => 1000,
            GridDefinitionTemplate::HovmollerDiagramGrid => 1100,
            GridDefinitionTemplate::TimeSectionGrid => 1200,
            GridDefinitionTemplate::Missing => 65535,
        }
    }
}

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum EarthShape {
    #[description = "Earth assumed spherical with radius = 6,367,470.0 m"]
    Spherical = 0,
    #[description = "Earth assumed spherical with radius specified (in m) by data producer"]
    SpecifiedRadiusSpherical = 1,
    #[description = "Earth assumed oblate spheroid with size as determined by IAU in 1965 (major axis = 6,378,160.0 m, minor axis = 6,356,775.0 m, f = 1/297.0) "]
    OblateIAU = 2,
    #[description = "Earth assumed oblate spheroid with major and minor axes specified (in km) by data producer"]
    OblateKM = 3,
    #[description = "Earth assumed oblate spheroid as defined in IAG-GRS80 model (major axis = 6,378,137.0 m, minor axis = 6,356,752.314 m, f = 1/298.257222101) "]
    OblateIAGGRS80 = 4,
    #[description = "Earth assumed represented by WGS84 (as used by ICAO since 1998) "]
    WGS84 = 5,
    #[description = "Earth assumed spherical with radius of 6,371,229.0 m"]
    Spherical2 = 6,
    #[description = "Earth assumed oblate spheroid with major and minor axes specified (in m) by data producer "]
    OblateM = 7,
    #[description = "Earth model assumed spherical with radius 6371200 m, but the horizontal datum of the resulting latitude/longitude field is the WGS84 reference frame"]
    OblateWGS84 = 8,
    Missing = 255,
}

pub struct LatitudeLongitudeGridTemplate<'a> {
    data: &'a[u8],
}

impl <'a> Template for LatitudeLongitudeGridTemplate<'a> {
    fn template_type(&self) -> TemplateType {
        TemplateType::Grid
    }

    fn template_number(&self) -> u16 {
        0
    }

    fn data(&self) -> &[u8] {
        self.data
    }

    fn template_name(&self) -> &str {
        "Latitude Longitude"
    }
}

impl <'a> LatitudeLongitudeGridTemplate<'a> {
    pub fn earth_shape(&self) -> EarthShape {
        self.data[14].into()
    }

    pub fn earth_radius_scale_factor(&self) -> u8 {
        self.data[15]
    }

    pub fn earth_radius_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 16).unwrap_or(0)
    }
    
    pub fn earth_major_axis_scale_factor(&self) -> u8 {
        self.data[20]
    }

    pub fn earth_major_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 21).unwrap_or(0)
    }
    
    pub fn earth_minor_axis_scale_factor(&self) -> u8 {
        self.data[25]
    }
    
    pub fn earth_minor_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 26).unwrap_or(0)
    }

    pub fn parallel_point_count(&self) -> u32 {
        read_u32_from_bytes(self.data, 30).unwrap_or(0)
    }
    
    pub fn meridian_point_count(&self) -> u32 {
        read_u32_from_bytes(self.data, 34).unwrap_or(0)
    }
    
    pub fn start_latitude(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 46).unwrap_or(0) as f64;
        value*(10f64.powf(-6.0))
    }
    
    pub fn start_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 50).unwrap_or(0) as f64;
        value*(10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> Vec<u8> {
        bit_array_from_bytes(&self.data[54..55])
    }

    pub fn end_latitude(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 55).unwrap_or(0) as f64;
        value*(10f64.powf(-6.0))
    }
    
    pub fn end_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 59).unwrap_or(0) as f64;
        value*(10f64.powf(-6.0))
    }
    
    pub fn i_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 63).unwrap_or(0) as f64;
        value*(10f64.powf(-6.0))
    }
    
    pub fn j_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 67).unwrap_or(0) as f64;
        value*(10f64.powf(-6.0))
    }

    pub fn scanning_mode_flags(&self) -> u8 {
        self.data[71]
    }
}