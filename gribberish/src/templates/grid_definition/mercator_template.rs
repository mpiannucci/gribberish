use mappers::Projection;

use crate::{
    error::GribberishError,
    templates::template::{Template, TemplateType},
    utils::{
        iter::projection::{
            GridProjection, LatLngProjection, ProjectedGrid, RegularCoordinateIterator,
        },
        projections::mercator::Mercator,
        read_u32_from_bytes,
    },
};

use super::{
    earth_shape::EarthShapeDefinition,
    tables::{ScanningMode, ScanningModeFlags},
    GridDefinitionTemplate,
};

pub struct MercatorTemplate {
    data: Vec<u8>,
}

impl Template for MercatorTemplate {
    fn template_type(&self) -> TemplateType {
        TemplateType::Grid
    }

    fn template_number(&self) -> u16 {
        10
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_name(&self) -> &str {
        "Mercator"
    }
}

impl MercatorTemplate {
    pub fn new(data: Vec<u8>) -> Self {
        MercatorTemplate { data }
    }

    fn earth(&self) -> EarthShapeDefinition<'_> {
        EarthShapeDefinition::new(&self.data)
    }

    pub fn number_of_points_on_x_axis(&self) -> u32 {
        read_u32_from_bytes(&self.data, 30).unwrap_or(0)
    }

    pub fn number_of_points_on_y_axis(&self) -> u32 {
        read_u32_from_bytes(&self.data, 34).unwrap_or(0)
    }

    pub fn latitude_of_first_grid_point(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 38).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    pub fn longitude_of_first_grid_point(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 42).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    /// `LaD`: the latitude at which the encoded grid lengths are true.
    pub fn latitude_of_true_scale(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 47).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    /// Exposed for callers; not used internally to cross-check the grid this
    /// template computes from the first point, step and count.
    pub fn latitude_of_last_grid_point(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 51).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    /// Exposed for callers; not used internally to cross-check the grid this
    /// template computes from the first point, step and count.
    pub fn longitude_of_last_grid_point(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 55).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[59])
    }

    /// Angle of rotation of the projection. This crate's Mercator projection
    /// implements only an unrotated Mercator, so a non-zero orientation is
    /// assumed not to occur; it is parsed but not validated, and a file that
    /// actually sets it would silently produce incorrect coordinates.
    pub fn grid_orientation(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 60).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    pub fn x_direction_grid_length(&self) -> f64 {
        read_u32_from_bytes(&self.data, 64).unwrap_or(0) as f64 * 1e-3
    }

    pub fn y_direction_grid_length(&self) -> f64 {
        read_u32_from_bytes(&self.data, 68).unwrap_or(0) as f64 * 1e-3
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

    /// Template 3.10 has no central-meridian field, so the projection origin is
    /// the prime meridian.
    pub fn projection(&self) -> Result<Mercator, GribberishError> {
        Ok(Mercator::new(
            0.0,
            self.latitude_of_true_scale(),
            self.earth().ellipsoid()?,
        ))
    }
}

impl GridDefinitionTemplate for MercatorTemplate {
    fn proj_name(&self) -> String {
        "merc".to_string()
    }

    fn proj_params(&self) -> std::collections::HashMap<String, f64> {
        let mut params = std::collections::HashMap::new();
        params.insert("lat_ts".to_string(), self.latitude_of_true_scale());
        params.insert("lon_0".to_string(), 0.0);

        let earth_params = self.earth().proj_params().unwrap_or_default();
        for (k, v) in earth_params {
            params.insert(k, v);
        }
        params
    }

    fn proj_string(&self) -> String {
        let earth_shape = self.earth().proj_string().unwrap_or("".to_string());
        format!(
            "+proj=merc lat_ts={} lon_0=0 {earth_shape}",
            self.latitude_of_true_scale()
        )
    }

    fn crs(&self) -> String {
        // The grid's parameters (arbitrary earth shape and latitude of true
        // scale) do not correspond to a standard EPSG code, so the CRS is
        // reported as unknown rather than guessed.
        "unknown".to_string()
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
        let projection = self.projection().expect("Invalid projection");
        let start_lng = wrap_longitude(self.longitude_of_first_grid_point());
        let (start_x, start_y) = projection
            .project(start_lng, self.latitude_of_first_grid_point())
            .expect("Failed to project mercator start coordinates");

        LatLngProjection::Projected(ProjectedGrid {
            x: RegularCoordinateIterator::new(start_x, self.x_step(), self.x_count()),
            y: RegularCoordinateIterator::new(start_y, self.y_step(), self.y_count()),
            projection: GridProjection::Mercator(projection),
            projection_name: self.proj_name(),
            projection_params: self.proj_params(),
        })
    }
}

fn wrap_longitude(lng: f64) -> f64 {
    if lng > 180.0 {
        lng - 360.0
    } else {
        lng
    }
}
