use gribberish::message::Message;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::exceptions::PyTypeError;

#[pyclass]
struct GribMessage {
    inner: Message,
}

#[pymethods]
impl GribMessage {
    #[getter]
    fn get_var_name(&self) -> PyResult<String> {
        match(self.inner.variable_name()) {
            Ok(v) => Ok(v), 
            Err(e) => Err(PyTypeError::new_err(e))
        }
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
        .map(|m| GribMessage{inner: m})
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
