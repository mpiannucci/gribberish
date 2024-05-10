use bitvec::prelude::*;

use super::grid_definition_template::GridDefinitionTemplate;
use super::tables::{EarthShape, ScanningModeFlags, ScanningMode};
use crate::templates::template::{Template, TemplateType};
use crate::utils::iter::projection::{RegularCoordinateIterator, LatLngProjection, PlateCareeProjection};
use crate::utils::read_u32_from_bytes;

use std::iter::Iterator;
use std::vec::Vec;


pub struct LatLngTemplate {
    data: Vec<u8>,
}

impl Template for LatLngTemplate {
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
        "Latitude Longitude: EPSG 4326"
    }
}

impl LatLngTemplate {
    pub fn new(data: Vec<u8>) -> Self {
        LatLngTemplate { data }
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

    pub fn parallel_point_count(&self) -> u32 {
        read_u32_from_bytes(&self.data, 30).unwrap_or(0)
    }

    pub fn meridian_point_count(&self) -> u32 {
        read_u32_from_bytes(&self.data, 34).unwrap_or(0)
    }

    pub fn basic_angle(&self) -> u32 {
        read_u32_from_bytes(&self.data, 38).unwrap_or(0)
    }

    pub fn subdivision(&self) -> u32 {
        read_u32_from_bytes(&self.data, 42).unwrap_or(0)
    }

    pub fn start_latitude(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 46).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn start_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 50).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> &BitSlice<u8, Msb0> {
        (&self.data[54..55]).view_bits()
    }

    pub fn end_latitude(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 55).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn end_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 59).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn i_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 63).unwrap_or(0) as f64;
        let value = value * (10f64.powf(-6.0));

        if self.scanning_mode_flags()[0] == ScanningMode::MinusI {
            value * -1.0
        } else {
            value
        }
    }

    pub fn j_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 67).unwrap_or(0) as f64;
        let value = value * (10f64.powf(-6.0));

        if self.scanning_mode_flags()[1] == ScanningMode::MinusJ {
            value * -1.0
        } else {
            value
        }
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[71])
    }

    pub fn latitudes(&self) -> Vec<f64> {
        let latitude_start = self.start_latitude();
        let latitude_step = self.j_direction_increment();
        (0..self.y_count())
            .map(|i| latitude_start + i as f64 * latitude_step)
            .collect()
    }

    pub fn longitudes(&self) -> Vec<f64> {
        let longitude_start = self.start_longitude();
        let longitude_step = self.i_direction_increment();
        (0..self.x_count())
            .map(|i| longitude_start + i as f64 * longitude_step)
            .collect()
    }

    pub fn grid_bounds(&self) -> ((f64, f64), (f64, f64)) {
        ((self.start_latitude(), self.start_longitude()), (self.end_latitude(), self.end_longitude()))
    }
}

impl GridDefinitionTemplate for LatLngTemplate {
    fn proj_name(&self) -> String {
        "latlon".to_string()
    }

    fn proj_params(&self) -> std::collections::HashMap<String, f64> {
        let mut params = std::collections::HashMap::new();
        params.insert("a".to_string(), 6367470.0);
        params.insert("b".to_string(), 6367470.0);
        params
    }

    fn proj_string(&self) -> String {
        format!("+proj=latlon +a=6367470 +b=6367470")
    }

    fn crs(&self) -> String {
        "EPSG:4326".to_string()
    }

    fn grid_point_count(&self) -> usize {
        (self.parallel_point_count() * self.meridian_point_count()) as usize
    }

    fn is_regular_grid(&self) -> bool {
        true
    }

    fn y_count(&self) -> usize {
        self.meridian_point_count() as usize
    }

    fn x_count(&self) -> usize {
        self.parallel_point_count() as usize
    }

    fn projector(&self) -> LatLngProjection {
        let lat_iter = RegularCoordinateIterator::new(
            self.start_latitude(),
            self.j_direction_increment(),
            self.y_count()
        );

        let lon_iter = RegularCoordinateIterator::new(
            self.start_longitude(),
            self.i_direction_increment(),
            self.x_count()
        );

        LatLngProjection::PlateCaree(PlateCareeProjection {
            latitudes: lat_iter, 
            longitudes: lon_iter,
            projection_name: self.proj_name(),
            projection_params: self.proj_params(),
        })
    }
}
