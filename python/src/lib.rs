use std::collections::HashMap;
use std::convert::TryFrom;

use gribberish::data_message::DataMessage;
use gribberish::message::{Message, read_messages, map_messages};
use numpy::{Ix2, PyArray, PyArray1, PyArray2, PyArray3, PyArrayDyn};
use pyo3::exceptions::PyTypeError;
use pyo3::types::PyDateTime;
use pyo3::wrap_pyfunction;
use pyo3::{prelude::*, types::PyTzInfo};
use ndarray::array;

#[pyclass]
struct GribMessage {
    inner: DataMessage,
}

#[pymethods]
impl GribMessage {
    #[getter]
    fn get_var_name(&self) -> &str {
        self.inner.name.as_str()
    }

    #[getter]
    fn get_var_abbrev(&self) -> &str {
        self.inner.var.as_str()
    }

    #[getter]
    fn get_units(&self) -> &str {
        self.inner.units.as_str()
    }

    #[getter]
    fn get_array_index(&self) -> Option<usize> {
        self.inner.array_index
    }

    #[getter]
    fn get_forecast_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        PyDateTime::from_timestamp(py, self.inner.forecast_date.timestamp() as f64, None)
    }

    #[getter]
    fn get_reference_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        PyDateTime::from_timestamp(py, self.inner.reference_date.timestamp() as f64, None)
    }

    #[getter]
    fn proj(&self) -> &str {
        self.inner.proj.as_str()
    }

    fn data<'py>(&self, py: Python<'py>) -> &'py PyArray<f64, Ix2> {
        PyArray::from_vec2(py, &self.inner.data).unwrap()
    }

    fn latitudes<'py>(&self, py: Python<'py>) -> &'py PyArray1<f64> {
        PyArray::from_slice(py, &self.inner.latitude)
    }

    fn longitudes<'py>(&self, py: Python<'py>) -> &'py PyArray1<f64> {
        PyArray::from_slice(py, &self.inner.longitude)
    }
}

#[pyfunction]
fn parse_grib_message<'py>(data: &[u8], offset: usize) -> PyResult<GribMessage> {
    match Message::from_data(&data.to_vec(), offset) {
        Some(m) => Ok(GribMessage { inner: DataMessage::try_from(m).unwrap() }),
        None => Err(PyTypeError::new_err("Failed to read GribMessage")),
    }
}

#[pyfunction]
fn parse_grib_messages(data: &[u8]) -> PyResult<Vec<GribMessage>> {
    let messages = read_messages(data)
        .map(|m| GribMessage { inner: DataMessage::try_from(m).unwrap() })
        .collect();

    Ok(messages)
}

#[pyfunction]
fn parse_grib_mapping(data: &[u8]) -> HashMap<String, (usize, usize)> {
    map_messages(data)
}

#[pymodule]
fn gribberish(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GribMessage>()?;
    m.add_function(wrap_pyfunction!(parse_grib_message, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_messages, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_mapping, m)?)?;
    Ok(())
}
