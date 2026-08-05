//! Shape of the earth, octets 15-30 of the grid definition section.
//!
//! These octets are byte-identical across grid definition templates 3.0, 3.10,
//! 3.20, and 3.30, so every template delegates here rather than repeating the
//! parsing and the proj/ellipsoid mapping.

use mappers::Ellipsoid;

use super::tables::EarthShape;
use crate::{error::GribberishError, utils::read_u32_from_bytes};

pub struct EarthShapeDefinition<'a> {
    data: &'a [u8],
}

impl<'a> EarthShapeDefinition<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn shape(&self) -> EarthShape {
        self.data[14].into()
    }

    pub fn radius_scale_factor(&self) -> u8 {
        self.data[15]
    }

    pub fn radius_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 16).unwrap_or(0)
    }

    pub fn major_axis_scale_factor(&self) -> u8 {
        self.data[20]
    }

    pub fn major_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 21).unwrap_or(0)
    }

    pub fn minor_axis_scale_factor(&self) -> u8 {
        self.data[25]
    }

    pub fn minor_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(self.data, 26).unwrap_or(0)
    }

    pub fn radius(&self) -> f64 {
        self.radius_scaled_value() as f64 * 10f64.powi(-(self.radius_scale_factor() as i32))
    }

    pub fn major_axis(&self) -> f64 {
        self.major_axis_scaled_value() as f64 * 10f64.powi(-(self.major_axis_scale_factor() as i32))
    }

    pub fn minor_axis(&self) -> f64 {
        self.minor_axis_scaled_value() as f64 * 10f64.powi(-(self.minor_axis_scale_factor() as i32))
    }

    pub fn ellipsoid(&self) -> Result<Ellipsoid, GribberishError> {
        match self.shape() {
            EarthShape::Spherical => Ok(Ellipsoid {
                A: 6_367_470.0,
                B: 6_367_470.0,
                E: 0.0,
                F: 0.0,
            }),
            EarthShape::SpecifiedRadiusSpherical => {
                let radius = self.radius();
                Ok(Ellipsoid {
                    A: radius,
                    B: radius,
                    E: 0.0,
                    F: 0.0,
                })
            }
            EarthShape::OblateIAU => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateIAU".into(),
            )),
            EarthShape::OblateKM => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateKM".into(),
            )),
            EarthShape::OblateIAGGRS80 => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateIAGGRS80".into(),
            )),
            EarthShape::WGS84 => Ok(Ellipsoid::WGS84),
            EarthShape::Spherical2 => Ok(Ellipsoid {
                A: 6_371_229.0,
                B: 6_371_229.0,
                E: 0.0,
                F: 0.0,
            }),
            EarthShape::OblateM => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateM".into(),
            )),
            EarthShape::OblateWGS84 => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateWGS84".into(),
            )),
            EarthShape::Missing => Err(GribberishError::GridTemplateError(
                "Missing EarthShape".into(),
            )),
        }
    }

    pub fn proj_string(&self) -> Result<String, GribberishError> {
        match self.shape() {
            EarthShape::Spherical => Ok(" +a=6367470 +b=6367470".to_string()),
            EarthShape::SpecifiedRadiusSpherical => {
                let radius = self.radius();
                Ok(format!(" +a={radius} +b={radius}"))
            }
            EarthShape::OblateIAU => Ok(" +a=6,378,160.0 b=6356775 +rf=297".to_string()),
            EarthShape::OblateKM => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateKM".into(),
            )),
            EarthShape::OblateIAGGRS80 => {
                Ok(" +a=6378137 +b=6356752.314 +rf=298.257222101".to_string())
            }
            EarthShape::WGS84 => Ok(" +ellps=WGS84".to_string()),
            EarthShape::Spherical2 => Ok(" +a=6371229 +b=6371229".to_string()),
            EarthShape::OblateM => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateM".into(),
            )),
            EarthShape::OblateWGS84 => Err(GribberishError::GridTemplateError(
                "unimplemented: OblateWGS84".into(),
            )),
            EarthShape::Missing => Err(GribberishError::GridTemplateError(
                "Missing EarthShape".into(),
            )),
        }
    }

    pub fn proj_params(&self) -> Result<Vec<(String, f64)>, String> {
        match self.shape() {
            EarthShape::Spherical => Ok(vec![
                ("a".to_string(), 6_367_470.0),
                ("b".to_string(), 6_367_470.0),
            ]),
            EarthShape::SpecifiedRadiusSpherical => {
                let radius = self.radius();
                Ok(vec![("a".to_string(), radius), ("b".to_string(), radius)])
            }
            EarthShape::OblateIAU => Err("unimplemented: OblateIAU".into()),
            EarthShape::OblateKM => Err("unimplemented: OblateKM".into()),
            EarthShape::OblateIAGGRS80 => Ok(vec![
                ("a".to_string(), 6_378_137.0),
                ("b".to_string(), 6_356_752.314),
            ]),
            EarthShape::WGS84 => Err("unimplemented: WGS84".into()),
            EarthShape::Spherical2 => Ok(vec![
                ("a".to_string(), 6_371_229.0),
                ("b".to_string(), 6_371_229.0),
            ]),
            EarthShape::OblateM => Err("unimplemented: OblateM".into()),
            EarthShape::OblateWGS84 => Err("unimplemented: OblateWGS84".into()),
            EarthShape::Missing => Err("Missing EarthShape".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EarthShapeDefinition;
    use crate::templates::grid_definition::tables::EarthShape;

    /// Section 3 bytes with only the earth-shape octets set. Octet 15 is index
    /// 14, so the buffer needs 30 bytes to cover the block.
    fn section_with_shape(shape: u8) -> Vec<u8> {
        let mut data = vec![0u8; 40];
        data[14] = shape;
        data
    }

    /// Shape 6 is the fixed 6,371,229 m sphere NCEP uses for the NAQFC grids.
    #[test]
    fn ncep_sphere_shape_six() {
        let data = section_with_shape(6);
        let earth = EarthShapeDefinition::new(&data);

        assert_eq!(earth.shape(), EarthShape::Spherical2);

        let ellipsoid = earth.ellipsoid().unwrap();
        assert_eq!(ellipsoid.A, 6_371_229.0);
        assert_eq!(ellipsoid.B, 6_371_229.0);
        assert_eq!(ellipsoid.E, 0.0);

        assert_eq!(earth.proj_string().unwrap(), " +a=6371229 +b=6371229");
    }

    /// Shape 5 is the one ellipsoidal shape the existing templates accept.
    #[test]
    fn wgs84_shape_five() {
        let data = section_with_shape(5);
        let earth = EarthShapeDefinition::new(&data);

        assert_eq!(earth.shape(), EarthShape::WGS84);
        assert!(earth.ellipsoid().unwrap().E > 0.0);
        assert_eq!(earth.proj_string().unwrap(), " +ellps=WGS84");
    }

    /// Shape 1 carries a producer-specified radius in octets 16-20.
    #[test]
    fn specified_radius_shape_one() {
        let mut data = section_with_shape(1);
        data[15] = 0; // scale factor
        data[16..20].copy_from_slice(&6_371_000u32.to_be_bytes());

        let earth = EarthShapeDefinition::new(&data);
        assert_eq!(earth.radius(), 6_371_000.0);
        assert_eq!(earth.ellipsoid().unwrap().A, 6_371_000.0);
    }

    /// Shapes the existing templates reject must keep rejecting.
    #[test]
    fn unsupported_shapes_still_error() {
        for shape in [3u8, 7, 8] {
            let data = section_with_shape(shape);
            assert!(EarthShapeDefinition::new(&data).ellipsoid().is_err());
        }
    }

    /// A missing earth shape must return an error rather than panicking, so a
    /// single malformed message cannot abort the read of a whole file.
    #[test]
    fn missing_shape_errors_instead_of_panicking() {
        let data = section_with_shape(255);
        let earth = EarthShapeDefinition::new(&data);
        assert_eq!(earth.shape(), EarthShape::Missing);
        assert!(earth.ellipsoid().is_err());
        assert!(earth.proj_string().is_err());
    }
}
