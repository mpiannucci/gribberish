use crate::{templates::template::{Template, TemplateType}, utils::{read_u32_from_bytes, bit_array_from_bytes}};

use super::tables::{EarthShape, ScanningModeFlags, ScanningMode, ProjectionCenterFlags, ProjectionCenter};

pub struct LambertConformalTemplate<'a> {
    data: &'a [u8],
}

impl <'a> Template for LambertConformalTemplate<'a> {
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
        "Lambert Confromal"
    }
}

impl <'a> LambertConformalTemplate<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        LambertConformalTemplate { data }
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

    pub fn number_of_points_on_axis(&self) -> u32 {
        read_u32_from_bytes(self.data, 30).unwrap_or(0)
    }

    pub fn number_of_points_on_y_axis(&self) -> u32 {
        read_u32_from_bytes(self.data, 34).unwrap_or(0)
    }

    pub fn latitude_of_first_grid_point(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 38).unwrap_or(0);
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_first_grid_point(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 42).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> Vec<u8> {
        bit_array_from_bytes(&self.data[46..47])
    }

    pub fn latitude_of_dx_dy(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 47).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_paralell_meridian_to_latitude_increase(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 51).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn x_direction_grid_length(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 55).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-3.0))
    }

    pub fn y_direction_grid_length(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 59).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-3.0))
    }

    pub fn projection_centre_flags(&self) -> ProjectionCenterFlags {
        ProjectionCenter::read_flags(self.data[63])
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[64])
    }

    pub fn latin_1(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 65).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn latin_2(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 69).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn latitude_of_southern_pole(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 73).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn longitude_of_southern_pole(&self) -> f64 {
        let raw_value = read_u32_from_bytes(self.data, 77).unwrap_or(0); 
        let value = as_signed!(raw_value, i32) as f64;
        value * (10f64.powf(-6.0))
    }
}