mod message;
mod dataset;

use message::GribMessage;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::dataset::build_grib_array;
use crate::dataset::parse_grib_dataset;
use crate::message::parse_grib_array;
use crate::message::parse_grib_mapping;
use crate::message::parse_grib_message;
use crate::message::parse_grib_message_metadata;

#[pymodule]
fn _gribberish_python(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<GribMessage>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(parse_grib_message_metadata, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_message, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_mapping, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_dataset, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_array, m)?)?;
    m.add_function(wrap_pyfunction!(build_grib_array, m)?)?;
    Ok(())
}
