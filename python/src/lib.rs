mod message;
mod dataset;

use message::GribMessage;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::dataset::parse_grid_dataset;
use crate::message::parse_grib_data;
use crate::message::parse_grib_mapping;
use crate::message::parse_grib_message;
use crate::message::parse_grib_messages;

#[pymodule]
fn gribberishpy(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GribMessage>()?;
    m.add_function(wrap_pyfunction!(parse_grib_message, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_messages, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_mapping, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grid_dataset, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_data, m)?)?;
    Ok(())
}