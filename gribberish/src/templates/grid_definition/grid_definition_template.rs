use std::collections::HashMap;

use crate::{error::GribberishError, utils::iter::projection::LatLngProjection};

pub trait GridDefinitionTemplate {
    fn proj_string(&self) -> String;
    fn proj_name(&self) -> String;
    fn proj_params(&self) -> HashMap<String, f64>;
    fn crs(&self) -> String;
    fn grid_point_count(&self) -> usize;
    fn is_regular_grid(&self) -> bool;
    fn y_count(&self) -> usize;
    fn x_count(&self) -> usize;
    /// Builds the coordinate projector for this grid. Fails when the grid's
    /// projection parameters are invalid or its first grid point cannot be
    /// projected, since no coordinates can be derived in either case.
    fn projector(&self) -> Result<LatLngProjection, GribberishError>;
}
