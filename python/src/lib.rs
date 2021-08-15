use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use gribberish::message::Message;


#[pyclass]
struct GribMessage{
    inner: Message,
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pymodule]
fn gribberish(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    Ok(())
}

