use mappers::Projection;

use crate::{
    error::GribberishError,
    templates::template::{Template, TemplateType},
    utils::{
        iter::projection::{
            GridProjection, LatLngProjection, ProjectedGrid, RegularCoordinateIterator,
        },
        projections::polar_stereographic::PolarStereographic,
        read_u32_from_bytes,
    },
};

use super::{
    earth_shape::EarthShapeDefinition,
    tables::{ProjectionCenter, ProjectionCenterFlags, ScanningMode, ScanningModeFlags},
    GridDefinitionTemplate,
};

pub struct PolarStereographicTemplate {
    data: Vec<u8>,
}

impl Template for PolarStereographicTemplate {
    fn template_type(&self) -> TemplateType {
        TemplateType::Grid
    }

    fn template_number(&self) -> u16 {
        20
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_name(&self) -> &str {
        "Polar Stereographic"
    }
}

impl PolarStereographicTemplate {
    pub fn new(data: Vec<u8>) -> Self {
        PolarStereographicTemplate { data }
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

    /// `LoV`: the meridian that runs straight down the map from the pole.
    pub fn orientation_of_the_grid(&self) -> f64 {
        let raw = read_u32_from_bytes(&self.data, 51).unwrap_or(0);
        as_signed!(raw, 32, i32) as f64 * 1e-6
    }

    pub fn x_direction_grid_length(&self) -> f64 {
        read_u32_from_bytes(&self.data, 55).unwrap_or(0) as f64 * 1e-3
    }

    pub fn y_direction_grid_length(&self) -> f64 {
        read_u32_from_bytes(&self.data, 59).unwrap_or(0) as f64 * 1e-3
    }

    pub fn projection_centre_flags(&self) -> ProjectionCenterFlags {
        ProjectionCenter::read_flags(self.data[63])
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[64])
    }

    /// Which pole lies on the projection plane, from bit 1 of the projection
    /// centre flags.
    pub fn is_south_polar(&self) -> bool {
        self.projection_centre_flags()[0] == ProjectionCenter::SouthPole
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

    pub fn projection(&self) -> Result<PolarStereographic, GribberishError> {
        Ok(PolarStereographic::new(
            wrap_longitude(self.orientation_of_the_grid()),
            self.latitude_of_true_scale(),
            self.is_south_polar(),
            self.earth().ellipsoid()?,
        ))
    }
}

impl GridDefinitionTemplate for PolarStereographicTemplate {
    fn proj_name(&self) -> String {
        "stere".to_string()
    }

    fn proj_params(&self) -> std::collections::HashMap<String, f64> {
        let pole_latitude = if self.is_south_polar() { -90.0 } else { 90.0 };

        let mut params = std::collections::HashMap::new();
        params.insert("lat_0".to_string(), pole_latitude);
        params.insert("lat_ts".to_string(), self.latitude_of_true_scale());
        params.insert("lon_0".to_string(), self.orientation_of_the_grid());

        let earth_params = self.earth().proj_params().unwrap_or_default();
        for (k, v) in earth_params {
            params.insert(k, v);
        }
        params
    }

    fn proj_string(&self) -> String {
        let pole_latitude = if self.is_south_polar() { -90.0 } else { 90.0 };
        let earth_shape = self.earth().proj_string().unwrap_or("".to_string());
        format!(
            "+proj=stere lat_0={} lat_ts={} lon_0={} {earth_shape}",
            pole_latitude,
            self.latitude_of_true_scale(),
            self.orientation_of_the_grid()
        )
    }

    fn crs(&self) -> String {
        // The grid's parameters (arbitrary earth shape, orientation and
        // latitude of true scale) do not correspond to a standard EPSG code,
        // so the CRS is reported as unknown rather than guessed.
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
            .expect("Failed to project polar stereographic start coordinates");

        LatLngProjection::Projected(ProjectedGrid {
            x: RegularCoordinateIterator::new(start_x, self.x_step(), self.x_count()),
            y: RegularCoordinateIterator::new(start_y, self.y_step(), self.y_count()),
            projection: GridProjection::PolarStereographic(projection),
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
