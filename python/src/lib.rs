use gribberish::message::Message;
use numpy::{PyArray, PyArray1, PyArrayDyn, Ix2};
use pyo3::exceptions::PyTypeError;
use pyo3::types::PyDateTime;
use pyo3::wrap_pyfunction;
use pyo3::{prelude::*, types::PyTzInfo};

#[pyclass]
struct GribMessage {
    inner: Message,
}

#[pymethods]
impl GribMessage {
    #[getter]
    fn get_var_name(&self) -> PyResult<String> {
        match self.inner.variable_name() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    #[getter]
    fn get_var_abbrev(&self) -> PyResult<String> {
        match self.inner.variable_abbrev() {
            Ok(v) => Ok(v),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    #[getter]
    fn get_units(&self) -> PyResult<String> {
        match self.inner.unit() {
            Ok(u) => Ok(u),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    #[getter]
    fn get_forecast_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        match self.inner.forecast_date() {
            Ok(d) => PyDateTime::from_timestamp(py, d.timestamp() as f64, None),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    #[getter]
    fn get_reference_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        match self.inner.reference_date() {
            Ok(d) => PyDateTime::from_timestamp(py, d.timestamp() as f64, None),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    #[getter]
    fn get_array_index(&self) -> PyResult<Option<usize>> {
        match self.inner.array_index() {
            Ok(i) => Ok(i),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    fn location_data_index(&self, lat: f64, lon: f64) -> PyResult<usize> {
        match self.inner.data_index_for_location(&(lat, lon)) {
            Ok(u) => Ok(u),
            Err(e) => Err(PyTypeError::new_err(e)),
        }
    }

    fn raw_data_array<'py>(&self, py: Python<'py>) -> &'py PyArray1<f64> {
        let data = self.inner.data().unwrap();
        PyArray1::from_vec(py, data)
    }

    fn data_at_location(&self, lat: f64, lon: f64) -> PyResult<f64> {
        match self.inner.data_at_location(&(lat, lon)) {
            Ok(u) => Ok(u),
            Err(_) => Ok(f64::NAN),
        }
    }

    fn data<'py>(&self, py: Python<'py>) -> &'py PyArray<f64, Ix2> {
        let data = self.inner.data_grid().unwrap();
        PyArray::from_vec2(py, &data).unwrap()
    }
}

#[pyfunction]
fn parse_grib_message(data: &[u8], offset: usize) -> PyResult<GribMessage> {
    match Message::parse(data, offset) {
        Ok(m) => Ok(GribMessage { inner: m }),
        Err(e) => Err(PyTypeError::new_err(e)),
    }
}

#[pyfunction]
fn parse_grib_messages(data: &[u8]) -> PyResult<Vec<GribMessage>> {
    let messages = Message::parse_all(data)
        .into_iter()
        .map(|m| GribMessage { inner: m })
        .collect();

    Ok(messages)
}

#[pymodule]
fn gribberish(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<GribMessage>()?;
    m.add_function(wrap_pyfunction!(parse_grib_message, m)?)?;
    m.add_function(wrap_pyfunction!(parse_grib_messages, m)?)?;
    Ok(())
}
