//! Map projections needed by GRIB2 grid definition templates that the
//! `mappers` crate does not provide.

pub mod conformal;
pub mod mercator;

/// Normalize a longitude in degrees to `[-180, 180)`, the convention the rest
/// of gribberish uses for projected grids.
pub(crate) fn wrap_longitude_degrees(lon: f64) -> f64 {
    let wrapped = (lon + 180.0).rem_euclid(360.0) - 180.0;
    // rem_euclid can return exactly 360.0 for tiny negative inputs.
    if wrapped >= 180.0 {
        wrapped - 360.0
    } else {
        wrapped
    }
}
