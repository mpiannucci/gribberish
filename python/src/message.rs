use std::collections::HashMap;
use std::convert::TryFrom;

use gribberish::message::Message;
use gribberish::message_metadata::{scan_message_metadata, MessageMetadata};
use numpy::{PyArray, PyArray1};
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyDateTime, PyList};

#[pyclass]
#[derive(Clone)]
pub struct GribMessageMetadata {
    inner: MessageMetadata,
}

#[pymethods]
impl GribMessageMetadata {
    #[getter]
    fn message_size(&self) -> usize {
        self.inner.message_size
    }
 
    #[getter]
    fn var_name(&self) -> &str {
        self.inner.name.as_str()
    }

    #[getter]
    fn var_abbrev(&self) -> &str {
        self.inner.var.as_str()
    }

    #[getter]
    fn units(&self) -> &str {
        self.inner.units.as_str()
    }

    #[getter]
    fn generating_process(&self) -> String {
        self.inner.generating_process.to_string()
    }

    #[getter]
    fn statistical_process(&self) -> Option<String> {
        self.inner
            .statistical_process
            .clone()
            .map(|p| p.to_string())
    }

    #[getter]
    fn level_type(&self) -> String {
        self.inner.first_fixed_surface_type.to_string()
    }

    #[getter]
    fn level_value(&self) -> Option<f64> {
        self.inner.first_fixed_surface_value
    }

    #[getter]
    fn second_fixed_surface_type(&self) -> String {
        self.inner.second_fixed_surface_type.to_string()
    }

    #[getter]
    fn second_fixed_surface_value(&self) -> Option<f64> {
        self.inner.second_fixed_surface_value
    }

    #[getter]
    fn reference_date<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDateTime>> {
        PyDateTime::from_timestamp(py, self.inner.reference_date.timestamp() as f64, None)
    }

    #[getter]
    fn forecast_date<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDateTime>> {
        PyDateTime::from_timestamp(py, self.inner.forecast_date.timestamp() as f64, None)
    }

    #[getter]
    fn forecast_date_end<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyDateTime>>> {
        if let Some(forecast_end_date) = self.inner.forecast_end_date {
            let timestamp =
                PyDateTime::from_timestamp(py, forecast_end_date.timestamp() as f64, None)?;
            Ok(Some(timestamp))
        } else {
            Ok(None)
        }
    }

    #[getter]
    fn proj(&self) -> &str {
        self.inner.proj.as_str()
    }

    #[getter]
    fn crs(&self) -> &str {
        self.inner.crs.as_str()
    }

    #[getter]
    fn is_regular_grid(&self) -> bool {
        self.inner.is_regular_grid
    }

    #[getter]
    fn grid_shape(&self) -> (usize, usize) {
        self.inner.grid_shape
    }

    #[getter]
    fn array_len(&self) -> usize {
        self.inner.data_point_count()
    }

    #[getter]
    fn spatial_dims<'py>(&self, _py: Python<'py>) -> Vec<String> {
        if self.inner.is_regular_grid {
            vec!["latitude".into(), "longitude".into()]
        } else {
            vec!["y".into(), "x".into()]
        }
    }

    #[getter]
    fn non_spatial_dims<'py>(&self, _py: Python<'py>) -> Vec<String> {
        if self.inner.first_fixed_surface_type.is_single_level() {
            vec!["time".into()]
        } else {
            vec![
                "time".into(),
                self.inner.first_fixed_surface_type.coordinate_name().into(),
            ]
        }
    }

    #[getter]
    fn dims<'py>(&self, py: Python<'py>) -> Vec<String> {
        let mut other_dims = self.non_spatial_dims(py);
        other_dims.append(&mut self.spatial_dims(py));
        other_dims
    }

    #[getter]
    fn dims_key<'py>(&self, py: Python<'py>) -> String {
        self.dims(py).join(":")
    }

    #[getter]
    fn non_dims_key<'py>(&self, py: Python<'py>) -> String {
        format!(
            "{var_name}:{non_dims}",
            var_name = self.inner.var.to_lowercase(),
            non_dims = self.non_spatial_dims(py).join(":")
        )
    }

    fn latlng<'py>(&self, py: Python<'py>) -> (Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>) {
        let (lat, lng) = self.inner.latlng();
        (PyArray::from_vec(py, lat), PyArray::from_vec(py, lng))
    }

    fn xy<'py>(&self, py: Python<'py>) -> (Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>) {
        let (x, y) = self.inner.xy();
        (PyArray::from_vec(py, x), PyArray::from_vec(py, y))
    }
}

#[pyclass]
pub struct GribMessage {
    offset: usize,
    raw_data: Vec<u8>,
    #[pyo3(get)]
    pub metadata: GribMessageMetadata,
}

#[pymethods]
impl GribMessage {
    fn data<'py>(&self, py: Python<'py>, ) -> Bound<'py, PyArray1<f64>> {
        parse_grib_array(py, &self.raw_data, self.offset)
    }
}

#[pyfunction]
pub fn parse_grib_array<'py>(py: Python<'py>, data: &[u8], offset: usize) -> Bound<'py, PyArray1<f64>> {
    let message = Message::from_data(data, offset).unwrap();
    let data = message.data().unwrap();
    PyArray::from_vec(py, data)
}

/// Parse multiple GRIB messages from a single buffer
/// Returns a list of numpy arrays
#[pyfunction]
pub fn parse_grib_array_batch<'py>(
    py: Python<'py>,
    data: &[u8],
    offsets: Vec<usize>
) -> Vec<Bound<'py, PyArray1<f64>>> {
    offsets
        .iter()
        .map(|&offset| {
            let message = Message::from_data(data, offset).unwrap();
            let msg_data = message.data().unwrap();
            PyArray::from_vec(py, msg_data)
        })
        .collect()
}

#[pyfunction]
pub fn parse_grib_message_metadata(data: &[u8], offset: usize) -> PyResult<GribMessageMetadata> {
    let message = Message::from_data(data, offset).unwrap();
    let metadata = MessageMetadata::try_from(&message).unwrap();
    Ok(GribMessageMetadata { inner: metadata })
}

#[pyfunction]
pub fn parse_grib_message<'py>(data: &[u8], offset: usize) -> PyResult<GribMessage> {
    match Message::from_data(&data.to_vec(), offset) {
        Some(m) => Ok(GribMessage {
            offset,
            raw_data: data.to_vec(),
            metadata: GribMessageMetadata {
                inner: MessageMetadata::try_from(&m).unwrap(),
            },
        }),
        None => Err(PyTypeError::new_err("Failed to read GribMessage")),
    }
}

#[pyfunction]
#[pyo3(signature = (data, drop_variables=None))]
pub fn parse_grib_mapping<'py>(
    data: &[u8],
    drop_variables: Option<Bound<'py, PyList>>,
) -> HashMap<String, (usize, usize, GribMessageMetadata)> {
    let drop_variables = if let Some(drop_variables) = drop_variables {
        drop_variables
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    scan_message_metadata(data)
        .into_iter()
        .filter_map(|(k, v)| {
            let message: GribMessageMetadata = GribMessageMetadata { inner: v.2 };

            if drop_variables.contains(&message.var_name().to_lowercase()) {
                None
            } else {
                // v is (index, byte_offset, metadata)
                // Return (byte_offset, message_size, metadata)
                Some((k.clone(), (v.1, message.inner.message_size, message)))
            }
        })
        .collect()
}
