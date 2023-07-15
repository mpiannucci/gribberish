use std::collections::HashMap;
use std::convert::TryFrom;

use gribberish::data_message::DataMessage;
use gribberish::message::{read_messages, Message};
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
    fn forecast_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        PyDateTime::from_timestamp(py, self.inner.forecast_date.timestamp() as f64, None)
    }

    #[getter]
    fn reference_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        PyDateTime::from_timestamp(py, self.inner.reference_date.timestamp() as f64, None)
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

    fn latlng<'py>(&self, py: Python<'py>) -> (&'py PyArray1<f64>, &'py PyArray1<f64>) {
        let (lat, lng) = self.inner.latlng();
        (PyArray::from_slice(py, &lat), PyArray::from_slice(py, &lng))
    }
}

#[pyclass]
pub struct GribMessage {
    inner: DataMessage,
    #[pyo3(get)]
    pub metadata: GribMessageMetadata,
}

#[pymethods]
impl GribMessage {
    fn data<'py>(&self, py: Python<'py>) -> &'py PyArray1<f64> {
        PyArray::from_slice(py, &self.inner.data)
    }
}

#[pyfunction]
pub fn parse_grib_array<'py>(py: Python<'py>, data: &[u8], offset: usize, shape: Vec<usize>) ->  &'py PyArray1<f64> {
    let message = Message::from_data(data, offset).unwrap();

    let mut data = message.data().unwrap();

    // Every grib chunk is going to be assumed to be the size of one entire message spatially
    let chunk_size = shape.iter().rev().take(2).product::<usize>();
    data.resize(chunk_size, 0.0);

    PyArray::from_slice(py, &data)
}

#[pyfunction]
pub fn parse_grib_message<'py>(data: &[u8], offset: usize) -> PyResult<GribMessage> {
    match Message::from_data(&data.to_vec(), offset) {
        Some(m) => Ok(GribMessage {
            inner: DataMessage::try_from(&m).unwrap(),
            metadata: GribMessageMetadata {
                inner: MessageMetadata::try_from(&m).unwrap(),
            },
        }),
        None => Err(PyTypeError::new_err("Failed to read GribMessage")),
    }
}

#[pyfunction]
pub fn parse_grib_messages(data: &[u8]) -> PyResult<Vec<GribMessage>> {
    let messages = read_messages(data)
        .map(|m| GribMessage {
            inner: DataMessage::try_from(&m).unwrap(),
            metadata: GribMessageMetadata {
                inner: MessageMetadata::try_from(&m).unwrap(),
            },
        })
        .collect();

    Ok(messages)
}

#[pyfunction]
pub fn parse_grib_mapping(
    data: &[u8],
    drop_variables: Option<&PyList>,
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
                Some((k.clone(), (v.0, v.1, message)))
            }
        })
        .collect()
}
