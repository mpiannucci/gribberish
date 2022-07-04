use super::template::{Template, TemplateType};
use crate::utils::{bit_array_from_bytes, read_signed_from_bytes, read_u32_from_bytes};
use gribberish_macros::{DisplayDescription, FromValue};
use std::iter::Iterator;
use std::vec::Vec;

pub trait GridDefinitionTemplate<'a> {
    fn proj_string(&self) -> String;
    fn grid_point_count(&self) -> usize;
    fn start(&self) -> (f64, f64);
    fn origin(&self) -> (f64, f64);
    fn end(&self) -> (f64, f64);
    fn latitude_count(&self) -> usize;
    fn longitude_count(&self) -> usize;
    fn latitude_resolution(&self) -> f64;
    fn longitude_resolution(&self) -> f64;
    fn latitudes(&self) -> Vec<f64>;
    fn longitudes(&self) -> Vec<f64>;
    fn locations(&self) -> Vec<(f64, f64)>;
    fn location_for_index(&self, index: usize) -> Result<(f64, f64), &'static str>;
    fn index_for_location(&self, latitude: f64, longitude: f64) -> Result<usize, &'static str>;

    fn latitude_for_indice(&self, indice: usize) -> Result<f64, &'static str>;
    fn longitude_for_indice(&self, indice: usize) -> Result<f64, &'static str>;
    fn location_for_indices(&self, indices: (usize, usize)) -> Result<(f64, f64), &'static str> {
        let lat = self.latitude_for_indice(indices.0)?;
        let lon = self.longitude_for_indice(indices.1)?;
        Ok((lat, lon))
    }

    fn indice_for_latitude(&self, latitude: f64) -> Result<usize, &'static str>;
    fn indice_for_longitude(&self, longitude: f64) -> Result<usize, &'static str>;
    fn indices_for_location(
        &self,
        latitude: f64,
        longitude: f64,
    ) -> Result<(usize, usize), &'static str> {
        let j = self.indice_for_latitude(latitude)?;
        let i = self.indice_for_longitude(longitude)?;
        Ok((j, i))
    }

    fn index_for_indices(&self, indices: (usize, usize)) -> usize {
        (indices.0 * self.longitude_count()) + indices.1
    }

    fn latitudes_in_range(&self, range: (f64, f64)) -> Vec<f64> {
        self.latitudes()
            .into_iter()
            .filter(|l| *l > range.0 && *l < range.1)
            .collect()
    }

    fn longitudes_in_range(&self, range: (f64, f64)) -> Vec<f64> {
        self.longitudes()
            .into_iter()
            .filter(|l| *l > range.0 && *l < range.1)
            .collect()
    }

    fn locations_in_range(
        &self,
        latitude_range: (f64, f64),
        longitude_range: (f64, f64),
    ) -> Vec<(f64, f64)> {
        self.locations()
            .into_iter()
            .filter(|l| {
                l.0 > latitude_range.0
                    && l.0 < latitude_range.1
                    && l.1 > longitude_range.0
                    && l.1 < longitude_range.1
            })
            .collect()
    }

    fn location_grid(&self) -> Vec<Vec<Vec<f64>>> {
        let longitudes = self.longitudes();
        let latitudes = self.latitudes();
        latitudes
            .iter()
            .map(|lat| longitudes
                .iter()
                .map(|lon| vec![*lat, *lon])
                .collect()
            )
            .collect()
    }

    fn zerod_location_grid(&self) -> Vec<Vec<f64>> {
        let longitudes = self.longitudes();
        self.latitudes()
            .into_iter()
            .map(|_| longitudes.iter().map(|_| 0.).collect())
            .collect()
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
    data: &'a [u8],
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
        "Latitude Longitude: EPSG 4326"
    }
}

impl <'a> LatitudeLongitudeGridTemplate<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        LatitudeLongitudeGridTemplate { data }
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

impl <'a> GridDefinitionTemplate<'a> for LatitudeLongitudeGridTemplate<'a> {
    fn proj_string(&self) -> String {
        format!("+proj=latlon +a=6367470 +b=6367470 +units=degrees")
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
