use gribberish_types::Parameter;

use self::{
    ecmwf::{ecmwf_local_category, ecmwf_local_parameter},
    hydrology::{hydrology_category, hydrology_parameter},
    land_surface::{land_surface_category, land_surface_parameter},
    meteorological::{meteorological_category, meteorological_parameter},
    mrms::{multiradar_category, multiradar_parameter},
    oceanographic::{oceanographic_category, oceanographic_parameter},
    space::{space_category, space_parameter},
};

pub mod ecmwf;
pub mod hydrology;
pub mod land_surface;
pub mod meteorological;
pub mod mrms;
pub mod oceanographic;
pub mod space;

#[allow(dead_code)]
pub trait ProductDiscipline {
    fn from_category_parameter(category: u8, parameter: u8) -> Self;

    fn parameter(&self) -> Option<Parameter>;
}

pub fn category(discipline: u8, category: u8) -> &'static str {
    match discipline {
        0 => meteorological_category(category),
        1 => hydrology_category(category),
        2 => land_surface_category(category),
        3 => space_category(category),
        10 => oceanographic_category(category),
        192 => ecmwf_local_category(category),
        209 => multiradar_category(category),
        _ => "",
    }
}

pub fn parameter(discipline: u8, category: u8, parameter: u8) -> Option<Parameter> {
    match discipline {
        0 => meteorological_parameter(category, parameter),
        1 => hydrology_parameter(category, parameter),
        2 => land_surface_parameter(category, parameter),
        3 => space_parameter(category, parameter),
        10 => oceanographic_parameter(category, parameter),
        192 => ecmwf_local_parameter(category, parameter),
        209 => multiradar_parameter(category, parameter),
        _ => None,
    }
}
