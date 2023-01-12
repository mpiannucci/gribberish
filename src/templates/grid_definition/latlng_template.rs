use super::grid_definition_template::GridDefinitionTemplate;
use super::tables::{EarthShape, ScanningModeFlags, ScanningMode};
use crate::templates::template::{Template, TemplateType};
use crate::utils::{bit_array_from_bytes, read_u32_from_bytes};

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
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn start_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 50).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> Vec<u8> {
        bit_array_from_bytes(&self.data[54..55])
    }

    pub fn end_latitude(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 55).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn end_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 59).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn i_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 63).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn j_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 67).unwrap_or(0) as f64;
        let value = value * (10f64.powf(-6.0));

        if self.is_descending_latitude() {
            value * -1.0
        } else {
            value
        }
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[71])
    }

    fn is_descending_latitude(&self) -> bool {
        self.start_latitude() > self.end_latitude()
    }

    fn latitudes(&self) -> Vec<f64> {
        let latitude_start = self.start_latitude();
        let latitude_step = self.j_direction_increment();
        (0..self.latitude_count())
            .map(|i| latitude_start + i as f64 * latitude_step)
            .collect()
    }

    fn longitudes(&self) -> Vec<f64> {
        let longitude_start = self.start_longitude();
        let longitude_step = self.i_direction_increment();
        (0..self.longitude_count())
            .map(|i| longitude_start + i as f64 * longitude_step)
            .collect()
    }
}

impl GridDefinitionTemplate for LatLngTemplate {
    fn proj_string(&self) -> String {
        format!("+proj=latlon +a=6367470 +b=6367470")
    }

    fn crs(&self) -> String {
        "EPSG:4326".to_string()
    }

    fn grid_point_count(&self) -> usize {
        (self.parallel_point_count() * self.meridian_point_count()) as usize
    }

    fn start(&self) -> (f64, f64) {
        (self.start_latitude(), self.start_longitude())
    }

    fn end(&self) -> (f64, f64) {
        (self.end_latitude(), self.end_longitude())
    }

    fn latitude_count(&self) -> usize {
        self.meridian_point_count() as usize
    }

    fn longitude_count(&self) -> usize {
        self.parallel_point_count() as usize
    }

    fn latlng(&self) -> Vec<(f64, f64)> {
        let latitudes = self.latitudes();
        let longitudes = self.longitudes();

        let mut locations = Vec::with_capacity(latitudes.len() * longitudes.len());
        for lat_i in 0..latitudes.len() {
            for lon_i in 0..longitudes.len() {
                locations.push((latitudes[lat_i], longitudes[lon_i]));
            }
        }

        return locations;
    }
}
