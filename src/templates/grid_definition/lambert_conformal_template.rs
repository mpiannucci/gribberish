use bitvec::prelude::*;

use mappers::{projections::LambertConformalConic, Ellipsoid, Projection};

use crate::{
    error::GribberishError, templates::template::{Template, TemplateType}, utils::{
        iter::projection::{
            LambertConformalConicProjection, LatLngProjection, RegularCoordinateIterator,
        },
        read_u32_from_bytes,
    }
};

use super::{
    tables::{
        EarthShape, ProjectionCenter, ProjectionCenterFlags, ScanningMode, ScanningModeFlags,
    },
    GridDefinitionTemplate,
};

pub struct LambertConformalTemplate {
    data: Vec<u8>,
}

impl Template for LambertConformalTemplate {
    fn template_type(&self) -> TemplateType {
        TemplateType::Grid
    }

    fn template_number(&self) -> u16 {
        0
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_name(&self) -> &str {
        "Lambert Confromal"
    }
}

impl LambertConformalTemplate {
    pub fn new(data: Vec<u8>) -> Self {
        LambertConformalTemplate { data }
    }

    pub fn earth_shape(&self) -> EarthShape {
        self.data[14].into()
    }

    pub fn earth_radius_scale_factor(&self) -> u8 {
        self.data[15]
    }

    pub fn earth_radius_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 16).unwrap_or(0)
    }

    pub fn earth_major_axis_scale_factor(&self) -> u8 {
        self.data[20]
    }

    pub fn earth_major_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 21).unwrap_or(0)
    }

    pub fn earth_minor_axis_scale_factor(&self) -> u8 {
        self.data[25]
    }

    pub fn earth_minor_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 26).unwrap_or(0)
    }

    pub fn earth_major_axis(&self) -> f64 {
        self.earth_major_axis_scaled_value() as f64
            * 10f64.powi(-(self.earth_major_axis_scale_factor() as i32))
    }

    pub fn earth_minor_axis(&self) -> f64 {
        self.earth_minor_axis_scaled_value() as f64
            * 10f64.powi(-(self.earth_minor_axis_scale_factor() as i32))
    }

    pub fn earth_ellipsoid(&self) -> Result<Ellipsoid, GribberishError> {
        let major = self.earth_major_axis();
        let minor = self.earth_minor_axis();

        match self.earth_shape() {
            EarthShape::Spherical => Ok(Ellipsoid {
                A: 6_367_470.0,
                B: 6_367_470.0,
                E: 0.0,
                F: 0.0,
            }),
            EarthShape::SpecifiedRadiusSpherical => Ok(Ellipsoid {
                A: major,
                B: minor,
                E: 0.0,
                F: 0.0,
            }),
            EarthShape::OblateIAU => Err(GribberishError::GridTemplateError("unimplemented: OblateIAU".into())),
            EarthShape::OblateKM => Err(GribberishError::GridTemplateError("unimplemented: OblateKM".into())),
            EarthShape::OblateIAGGRS80 => Err(GribberishError::GridTemplateError("unimplemented: OblateIAGGRS80".into())),
            EarthShape::WGS84 => Ok(Ellipsoid::WGS84),
            EarthShape::Spherical2 => Ok(Ellipsoid {
                A: 6_371_229.0,
                B: 6_371_229.0,
                E: 0.0,
                F: 0.0,
            }),
            EarthShape::OblateM => Err(GribberishError::GridTemplateError("unimplemented: OblateM".into())),
            EarthShape::OblateWGS84 => Err(GribberishError::GridTemplateError("unimplemented: OblateWGS84".into())),
            EarthShape::Missing => Err(GribberishError::GridTemplateError("Missing EarthShape".into())),
        }
    }

    pub fn earth_proj_string(&self) -> Result<String, GribberishError> {
        let major = self.earth_major_axis();
        let minor = self.earth_minor_axis();

        match self.earth_shape() {
            EarthShape::Spherical => Ok(" +a=6367470 +b=6367470".to_string()),
            EarthShape::SpecifiedRadiusSpherical => Ok(format!(" +a={major} +b={minor}")),
            EarthShape::OblateIAU => Ok(" +a=6,378,160.0 b=6356775 +rf=297".to_string()),
            EarthShape::OblateKM => Err(GribberishError::GridTemplateError("unimplemented: OblateKM".into())),
            EarthShape::OblateIAGGRS80 => Ok(format!(" +a=6378137 +b=6356752.314 +rf=298.257222101")),
            EarthShape::WGS84 => Ok(" +ellps=WGS84".to_string()),
            EarthShape::Spherical2 => Ok(" +a=6371229 +b=6371229".to_string()),
            EarthShape::OblateM => Err(GribberishError::GridTemplateError("unimplemented: OblateM".into())),
            EarthShape::OblateWGS84 => Err(GribberishError::GridTemplateError("unimplemented: OblateWGS84".into())),
            EarthShape::Missing => todo!(),
        }
    }

    pub fn earth_proj_params(&self) -> Result<Vec<(String, f64)>, String> {
        let major = self.earth_major_axis();
        let minor = self.earth_minor_axis();

        match self.earth_shape() {
            EarthShape::Spherical => Ok(vec![("a".to_string(), 6_367_470.0), ("b".to_string(), 6_367_470.0)]),
            EarthShape::SpecifiedRadiusSpherical => Ok(vec![("a".to_string(), major), ("b".to_string(), minor)]),
            EarthShape::OblateIAU => Err("unimplemented: OblateIAU".into()),
            EarthShape::OblateKM => Err("unimplemented: OblateKM".into()),
            EarthShape::OblateIAGGRS80 => Ok(vec![("a".to_string(), 6_378_137.0), ("b".to_string(), 6_356_752.314)]),
            EarthShape::WGS84 => Err("unimplemented: WGS84".into()),
            EarthShape::Spherical2 => Ok(vec![("a".to_string(), 6_371_229.0), ("b".to_string(), 6_371_229.0)]),
            EarthShape::OblateM => Err("unimplemented: OblateM".into()),
            EarthShape::OblateWGS84 => Err("unimplemented: OblateWGS84".into()),
            EarthShape::Missing => Err("Missing EarthShape".into()),
        }
    }

    pub fn number_of_points_on_x_axis(&self) -> u32 {
        read_u32_from_bytes(&self.data, 30).unwrap_or(0)
    }

    pub fn number_of_points_on_y_axis(&self) -> u32 {
        read_u32_from_bytes(&self.data, 34).unwrap_or(0)
    }

    pub fn latitude_of_first_grid_point(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 38).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_first_grid_point(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 42).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> &BitSlice<u8, Msb0> {
        (&self.data[46..47]).view_bits()
    }

    pub fn latitude_of_dx_dy(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 47).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_paralell_meridian_to_latitude_increase(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 51).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn x_direction_grid_length(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 55).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-3.0))
    }

    pub fn y_direction_grid_length(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 59).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-3.0))
    }

    pub fn projection_centre_flags(&self) -> ProjectionCenterFlags {
        ProjectionCenter::read_flags(self.data[63])
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[64])
    }

    pub fn latin_1(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 65).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn latin_2(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 69).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn latitude_of_southern_pole(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 73).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_southern_pole(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 77).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn x_step(&self) -> f64 {
        if self.scanning_mode_flags()[0] == ScanningMode::PlusI {
            self.x_direction_grid_length()
        } else {
            -self.x_direction_grid_length()
        }
    }

    pub fn y_step(&self) -> f64 {
        if self.scanning_mode_flags()[1] == ScanningMode::PlusJ {
            self.y_direction_grid_length()
        } else {
            -self.y_direction_grid_length()
        }
    }

    pub fn projection(&self) -> Result<LambertConformalConic, GribberishError> {
        let mut lng = self.longitude_of_paralell_meridian_to_latitude_increase();
        lng = if lng > 180.0 { lng - 360.0 } else { lng };

        let earth_shape = self.earth_ellipsoid()?;

        LambertConformalConic::new(
            lng,
            self.latitude_of_dx_dy(),
            self.latin_1(),
            self.latin_2(),
            earth_shape,
        )
        .map_err(|e| GribberishError::GridTemplateError(format!("Failed to create lambert conformal conic projection: {e}")))
    }

    pub fn project_axes(&self) -> Result<(LambertConformalConic, Vec<f64>, Vec<f64>), GribberishError> {
        let mut start_lng = self.longitude_of_first_grid_point();
        start_lng = if start_lng > 180.0 {
            start_lng - 360.0
        } else {
            start_lng
        };

        let projection = self.projection()?;
        let (start_x, start_y) = projection
            .project(start_lng, self.latitude_of_first_grid_point())
            .map_err(|e| {
                GribberishError::GridTemplateError(format!(
                    "Failed to project start coordinates to lambert conformal conic coords: {e}"
                ))
            })?;

        let dx = if self.scanning_mode_flags()[0] == ScanningMode::PlusI {
            self.x_direction_grid_length()
        } else {
            -self.x_direction_grid_length()
        };
        let dy = if self.scanning_mode_flags()[1] == ScanningMode::PlusJ {
            self.y_direction_grid_length()
        } else {
            -self.y_direction_grid_length()
        };

        let x = (0..self.number_of_points_on_x_axis())
            .map(|i| start_x + dx * i as f64)
            .collect();

        let y = (0..self.number_of_points_on_y_axis())
            .map(|i| start_y + dy * i as f64)
            .collect();

        Ok((projection, x, y))
    }
}

impl GridDefinitionTemplate for LambertConformalTemplate {
    fn proj_name(&self) -> String {
        "lcc".to_string()
    }

    fn proj_params(&self) -> std::collections::HashMap<String, f64> {
        let mut params = std::collections::HashMap::new();
        params.insert(
            "lon_0".to_string(),
            self.longitude_of_paralell_meridian_to_latitude_increase(),
        );
        params.insert("lat_0".to_string(), self.latitude_of_dx_dy());
        params.insert("lat_1".to_string(), self.latin_1());
        params.insert("lat_2".to_string(), self.latin_2());

        let earth_params = self.earth_proj_params().unwrap_or_default();
        for (k, v) in earth_params {
            params.insert(k, v);
        }
        params
    }

    fn proj_string(&self) -> String {
        let earth_shape = self.earth_proj_string().unwrap_or("".to_string());
        format!(
            "+proj=lcc lon_0={} lat_0={} lat_1={} lat_2={} {earth_shape}",
            self.longitude_of_paralell_meridian_to_latitude_increase(),
            self.latitude_of_dx_dy(),
            self.latin_1(),
            self.latin_2()
        )
    }

    fn crs(&self) -> String {
        // This is probably not right
        "EPSG:2154".to_string()
    }

    fn grid_point_count(&self) -> usize {
        (self.number_of_points_on_x_axis() * self.number_of_points_on_y_axis()) as usize
    }

    fn is_regular_grid(&self) -> bool {
        false
    }

    fn y_count(&self) -> usize {
        self.number_of_points_on_y_axis() as usize
    }

    fn x_count(&self) -> usize {
        self.number_of_points_on_x_axis() as usize
    }

    fn projector(&self) -> LatLngProjection {
        let mut start_lng = self.longitude_of_first_grid_point();
        start_lng = if start_lng > 180.0 {
            start_lng - 360.0
        } else {
            start_lng
        };

        let projection = self.projection().expect("Invalid projection");

        let (start_x, start_y) = projection
            .project(start_lng, self.latitude_of_first_grid_point())
            .map_err(|e| {
                format!(
                    "Failed to project start coordinates to lambert conformal conic coords: {e}"
                )
            })
            .expect("Failed to project");

        let y_iter = RegularCoordinateIterator::new(
            start_y,
            self.y_step(),
            self.number_of_points_on_y_axis() as usize,
        );

        let x_iter = RegularCoordinateIterator::new(
            start_x,
            self.x_step(),
            self.number_of_points_on_x_axis() as usize,
        );

        LatLngProjection::LambertConformal(LambertConformalConicProjection {
            x: x_iter,
            y: y_iter,
            projection,
            projection_name: self.proj_name(),
            projection_params: self.proj_params(),
        })
    }
}
