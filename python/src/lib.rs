use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// A Python module implemented in Rust.
#[pymodule]
fn gribberish(py: Python, m: &PyModule) -> PyResult<()> {
    //m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    Ok(())
}

