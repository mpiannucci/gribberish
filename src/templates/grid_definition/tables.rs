use gribberish_macros::{DisplayDescription, FromValue};

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
