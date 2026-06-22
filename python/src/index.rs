use std::collections::BTreeMap;

use gribberish::index::{parse_index, IndexEntry};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDateTime;

#[pyclass]
#[derive(Clone)]
pub struct GribIndexEntry {
    inner: IndexEntry,
}

#[pymethods]
impl GribIndexEntry {
    #[getter]
    fn message_number(&self) -> usize {
        self.inner.message_number
    }

    #[getter]
    fn submessage(&self) -> Option<usize> {
        self.inner.submessage
    }

    #[getter]
    fn offset(&self) -> u64 {
        self.inner.offset
    }

    #[getter]
    fn length(&self) -> Option<u64> {
        self.inner.length
    }

    #[getter]
    fn reference_date<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDateTime>>> {
        self.inner
            .reference_date
            .map(|d| PyDateTime::from_timestamp(py, d.timestamp() as f64, None))
            .transpose()
    }

    #[getter]
    fn var(&self) -> Option<&str> {
        self.inner.var.as_deref()
    }

    #[getter]
    fn level(&self) -> Option<&str> {
        self.inner.level.as_deref()
    }

    #[getter]
    fn forecast_time(&self) -> Option<&str> {
        self.inner.forecast_time.as_deref()
    }

    #[getter]
    fn extra(&self) -> Vec<String> {
        self.inner.extra.clone()
    }

    #[getter]
    fn keys(&self) -> BTreeMap<String, String> {
        self.inner.keys.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "GribIndexEntry(message_number={}, offset={}, length={:?}, var={:?}, level={:?}, forecast_time={:?})",
            self.inner.message_number,
            self.inner.offset,
            self.inner.length,
            self.inner.var,
            self.inner.level,
            self.inner.forecast_time,
        )
    }
}

/// Parse a GRIB sidecar index file (NOAA wgrib2 `.idx` or ECMWF open-data
/// `.index`, auto-detected) into a list of entries locating each message.
/// `file_size` (if known) sizes the final entry of a NOAA index.
#[pyfunction]
#[pyo3(signature = (data, file_size=None))]
pub fn parse_grib_index(data: &str, file_size: Option<u64>) -> PyResult<Vec<GribIndexEntry>> {
    let entries = parse_index(data, file_size).map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(entries
        .into_iter()
        .map(|inner| GribIndexEntry { inner })
        .collect())
}
