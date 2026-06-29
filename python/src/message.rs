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
    fn perturbation_number(&self) -> Option<u8> {
        self.inner.perturbation_number
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

    #[pyo3(signature = (adjust_longitude_range=false, north_up=false))]
    fn latlng<'py>(
        &self,
        py: Python<'py>,
        adjust_longitude_range: bool,
        north_up: bool,
    ) -> (Bound<'py, PyArray1<f64>>, Bound<'py, PyArray1<f64>>) {
        let (lat, lng) = self.inner.latlng_adjusted(adjust_longitude_range, north_up);
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
    fn data<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        parse_grib_array(py, &self.raw_data, self.offset, false, false)
    }
}

#[pyfunction]
#[pyo3(signature = (data, offset, adjust_longitude_range=false, north_up=false))]
pub fn parse_grib_array<'py>(
    py: Python<'py>,
    data: &[u8],
    offset: usize,
    adjust_longitude_range: bool,
    north_up: bool,
) -> PyResult<Bound<'py, PyArray1<f64>>> {
    let message = Message::from_data(data, offset)
        .ok_or_else(|| PyTypeError::new_err("Failed to read GRIB message"))?;
    let values = message
        .data()
        .map_err(|e| PyTypeError::new_err(format!("Failed to decode GRIB data: {e}")))?;
    // Two independent, composable on-the-fly adjustments, both decided by the
    // projector so data and coordinates stay aligned:
    //   - adjust_longitude_range rolls the data columns so longitudes run
    //     -180..180 monotonically, matching the wrapped longitude coordinate;
    //   - north_up reverses the data rows so row 0 is the northern-most,
    //     matching the flipped latitude/y coordinate.
    // Each is a no-op on ineligible grids; build the projector only when needed.
    let values = if adjust_longitude_range || north_up {
        let projector = message
            .latlng_projector()
            .map_err(|e| PyTypeError::new_err(format!("Failed to build projection: {e}")))?;
        let values = projector.adjust_data_longitude(values, adjust_longitude_range);
        projector.adjust_data_north_up(values, north_up)
    } else {
        values
    };
    Ok(PyArray::from_vec(py, values))
}

/// Wrap a global 0–360° longitude coordinate to a monotonic −180…180° axis,
/// matching the data roll the codec applies. A no-op for grids that don't span
/// the globe. Used by the VirtualiZarr parser to rewrap the inlined longitude
/// coordinate at the boundary, keeping the eager dataset reader projection-faithful.
#[pyfunction]
pub fn adjust_longitude_values(py: Python<'_>, longitudes: Vec<f64>) -> Bound<'_, PyArray1<f64>> {
    PyArray::from_vec(py, gribberish::adjust_longitude_values(longitudes))
}

/// Reverse an ascending (south-first) 1-D latitude coordinate to a descending
/// north-first axis, matching the row flip the codec applies to each data chunk
/// when ``north_up`` is set. A no-op for an axis that already runs north-to-south.
/// Used by the VirtualiZarr parser to flip the inlined 1-D latitude coordinate
/// at the boundary, keeping the eager dataset reader projection-faithful.
#[pyfunction]
pub fn adjust_latitude_values(py: Python<'_>, latitudes: Vec<f64>) -> Bound<'_, PyArray1<f64>> {
    PyArray::from_vec(py, gribberish::adjust_latitude_values(latitudes))
}

#[pyfunction]
pub fn parse_grib_message_metadata(data: &[u8], offset: usize) -> PyResult<GribMessageMetadata> {
    let message = Message::from_data(data, offset)
        .ok_or_else(|| PyTypeError::new_err("Failed to read GRIB message"))?;
    let metadata = MessageMetadata::try_from(&message)
        .map_err(|e| PyTypeError::new_err(format!("Failed to parse metadata: {e}")))?;
    Ok(GribMessageMetadata { inner: metadata })
}

#[pyfunction]
pub fn parse_grib_message(data: &[u8], offset: usize) -> PyResult<GribMessage> {
    match Message::from_data(data, offset) {
        Some(m) => {
            let metadata = MessageMetadata::try_from(&m)
                .map_err(|e| PyTypeError::new_err(format!("Failed to parse metadata: {e}")))?;
            Ok(GribMessage {
                offset,
                raw_data: data.to_vec(),
                metadata: GribMessageMetadata { inner: metadata },
            })
        }
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
                Some((k.clone(), (v.0, v.1, message)))
            }
        })
        .collect()
}
