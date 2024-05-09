use gribberish_types::Parameter;

use self::{meteorological::{meteorological_category, meteorological_parameter}, land_surface::{land_surface_category, land_surface_parameter}, oceanographic::{oceanographic_category, oceanographic_parameter}, mrms::{multiradar_category, multiradar_parameter}};

pub mod land_surface;
pub mod meteorological;
pub mod mrms;
pub mod oceanographic;

#[allow(dead_code)]
pub trait ProductDiscipline {
    fn from_category_parameter(category: u8, parameter: u8) -> Self;

    fn parameter(&self) -> Option<Parameter>;
}

pub fn category(discipline: u8, category: u8) -> &'static str {
    match discipline {
        0 => meteorological_category(category),
        2 => land_surface_category(category),
        10 => oceanographic_category(category),
        209 => multiradar_category(category),
        _ => "",
    }
}

pub fn parameter(discipline: u8, category: u8, parameter: u8) -> Option<Parameter> {
    match discipline {
        0 => meteorological_parameter(category, parameter),
        2 => land_surface_parameter(category, parameter),
        10 => oceanographic_parameter(category, parameter),
        209 => multiradar_parameter(category, parameter),
        _ => None,
    }
}
