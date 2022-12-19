use super::grid_definition_template::GridDefinitionTemplate;
use super::tables::EarthShape;
use crate::templates::template::{Template, TemplateType};
use crate::utils::{bit_array_from_bytes, read_signed_from_bytes, read_u32_from_bytes};

use std::iter::Iterator;
use std::vec::Vec;


pub struct LatLngGridTemplate<'a> {
    data: &'a [u8],
}

impl <'a> Template for LatLngGridTemplate<'a> {
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
        "Latitude Longitude: EPSG 4326"
    }
}

impl <'a> LatLngGridTemplate<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        LatLngGridTemplate { data }
    }

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

    pub fn basic_angle(&self) -> u32 {
        read_u32_from_bytes(self.data, 38).unwrap_or(0)
    }

    pub fn subdivision(&self) -> u32 {
        read_u32_from_bytes(self.data, 42).unwrap_or(0)
    }

    pub fn start_latitude(&self) -> f64 {
        let value = read_signed_from_bytes(self.data, 46).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn start_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 50).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> Vec<u8> {
        bit_array_from_bytes(&self.data[54..55])
    }

    pub fn end_latitude(&self) -> f64 {
        let value = read_signed_from_bytes(self.data, 55).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn end_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 59).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn i_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 63).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn j_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(self.data, 67).unwrap_or(0) as f64;
        let value = value * (10f64.powf(-6.0));

        if self.is_descending_latitude() {
            value * -1.0
        } else {
            value
        }
    }

    pub fn scanning_mode_flags(&self) -> u8 {
        self.data[71]
    }

    fn is_descending_latitude(&self) -> bool {
        self.start_latitude() > self.end_latitude()
    }
}

impl <'a> GridDefinitionTemplate<'a> for LatLngGridTemplate<'a> {
    fn proj_string(&self) -> String {
        format!("+proj=latlon +a=6367470 +b=6367470")
    }

    fn grid_point_count(&self) -> usize {
        (self.parallel_point_count() * self.meridian_point_count()) as usize
    }

    fn start(&self) -> (f64, f64) {
        (self.start_latitude(), self.start_longitude())
    }

    fn origin(&self) -> (f64, f64) {
        let lat = (self.start_latitude() + self.end_latitude()) * 0.5;
        let lng = (self.start_longitude() + self.end_longitude()) * 0.5;
        (lat, lng)
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

    fn latitude_resolution(&self) -> f64 {
        self.j_direction_increment()
    }

    fn longitude_resolution(&self) -> f64 {
        self.i_direction_increment()
    }

    fn latitudes(&self) -> Vec<f64> {
        let latitude_start = self.start_latitude();
        let latitude_step = self.latitude_resolution();
        (0..self.latitude_count())
            .map(|i| latitude_start + i as f64 * latitude_step)
            .collect()
    }

    fn longitudes(&self) -> Vec<f64> {
        let longitude_start = self.start_longitude();
        let longitude_step = self.longitude_resolution();
        (0..self.longitude_count())
            .map(|i| longitude_start + i as f64 * longitude_step)
            .collect()
    }

    fn locations(&self) -> Vec<(f64, f64)> {
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

    fn index_for_location(&self, latitude: f64, longitude: f64) -> Result<usize, &'static str> {
        let descending = self.is_descending_latitude();
        if !descending && (latitude < self.start_latitude() || latitude > self.end_latitude()) {
            return Err("Latitude is out of range");
        } else if descending && (latitude > self.start_latitude() || latitude < self.end_latitude())
        {
            return Err("Latitude is out of range");
        } else if longitude < self.start_longitude() || longitude > self.end_longitude() {
            return Err("Longitude is out of range");
        }

        let lat_difference = (latitude - self.start_latitude()).abs();
        let lat_index = (lat_difference / self.latitude_resolution()).abs().round() as usize;

        let lon_difference = (longitude - self.start_longitude()).abs();
        let lon_index = (lon_difference / self.longitude_resolution()).abs().round() as usize;

        let index = lat_index * self.longitude_count() as usize + lon_index;
        Ok(index)
    }

    fn location_for_index(&self, index: usize) -> Result<(f64, f64), &'static str> {
        if index >= self.grid_point_count() {
            return Err("Index out of range");
        }

        let lat_index = index / self.latitude_resolution() as usize;
        let lon_index = index % self.longitude_resolution() as usize;

        let latitude = self.start_latitude() + self.latitude_resolution() * lat_index as f64;
        let longitude = self.start_longitude() + self.longitude_resolution() * lon_index as f64;

        Ok((latitude, longitude))
    }

    fn indice_for_latitude(&self, latitude: f64) -> Result<usize, &'static str> {
        let descending = self.is_descending_latitude();
        if !descending && (latitude < self.start_latitude() || latitude > self.end_latitude()) {
            return Err("Latitude is out of range");
        } else if descending && (latitude > self.start_latitude() || latitude < self.end_latitude())
        {
            return Err("Latitude is out of range");
        }

        let lat_difference = (latitude - self.start_latitude()).abs();
        let lat_index = (lat_difference / self.latitude_resolution()).abs().round() as usize;

        Ok(lat_index)
    }

    fn indice_for_longitude(&self, longitude: f64) -> Result<usize, &'static str> {
        if longitude < self.start_longitude() || longitude > self.end_longitude() {
            return Err("Longitude is out of range");
        }

        let lon_difference = (longitude - self.start_longitude()).abs();
        let lon_index = (lon_difference / self.longitude_resolution()).abs().round() as usize;

        Ok(lon_index)
    }

    fn latitude_for_indice(&self, indice: usize) -> Result<f64, &'static str> {
        if indice >= self.latitude_count() {
            return Err("Indice is out of range");
        }

        Ok(self.start_latitude() + self.latitude_resolution() * indice as f64)
    }

    fn longitude_for_indice(&self, indice: usize) -> Result<f64, &'static str> {
        if indice >= self.longitude_count() {
            return Err("Indice is out of range.");
        }

        Ok(self.start_longitude() + self.longitude_resolution() * indice as f64)
    }
}
