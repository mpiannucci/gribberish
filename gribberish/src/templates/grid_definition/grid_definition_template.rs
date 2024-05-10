use std::collections::HashMap;

use crate::utils::iter::projection::LatLngProjection;

pub trait GridDefinitionTemplate {
    fn proj_string(&self) -> String;
    fn proj_name(&self) -> String;
    fn proj_params(&self) -> HashMap<String, f64>;
    fn crs(&self) -> String;
    fn grid_point_count(&self) -> usize;
    fn is_regular_grid(&self) -> bool;
    fn y_count(&self) -> usize;
    fn x_count(&self) -> usize;
    fn projector(&self) -> LatLngProjection;
}
