use std::collections::{HashMap, HashSet};

use gribberish::message_metadata::{scan_message_metadata, MessageMetadata};
use numpy::{PyArray, PyArray1};
use pyo3::{
    prelude::*,
    types::{PyDateTime, PyDict, PyList},
};

#[pyclass]
pub struct GribCoord {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub dims: Vec<String>,
    pub raw_values: Vec<f64>,
    #[pyo3(get)]
    pub metadata: HashMap<String, String>,
}

#[pymethods]
impl GribCoord {
    #[getter]
    fn get_values<'py>(&self, py: Python<'py>) -> &'py PyArray1<f64> {
        PyArray::from_slice(py, &self.raw_values)
    }
}

#[pyclass]
pub struct GribDataArray {
    _offset: usize,
    _metadata: MessageMetadata,
}

#[pyclass]
pub struct GribDataset {
    mapping: HashMap<String, (usize, usize, MessageMetadata)>,
    var_names: Vec<String>,
    var_mapping: HashMap<String, Vec<String>>,
    // spatial_dims: Vec<String>,
    // non_spatial_dims: Vec<String>,
}

#[pymethods]
impl GribDataset {
    #[new]
    fn new(data: &[u8], drop_variables: Option<&PyList>) -> Self {
        let drop_variables = if let Some(drop_variables) = drop_variables {
            drop_variables
                .iter()
                .map(|d| d.to_string().to_lowercase())
                .collect::<Vec<String>>()
        } else {
            Vec::new()
        };

        let mapping = scan_message_metadata(data, true)
            .into_iter()
            .filter_map(|(k, v)| {
                if drop_variables.contains(&v.2.var.to_lowercase()) {
                    None
                } else {
                    Some((k.clone(), v))
                }
            })
            .collect::<HashMap<_, _>>();

        let mut vars: HashMap<String, HashSet<String>> = HashMap::new();
        let mut hash_mapping: HashMap<String, Vec<String>> = HashMap::new();

        for (k, v) in mapping.iter() {
            let var = v.2.var.clone();
            let hash = format!(
                "{surf}_{stat}{gen}",
                surf = v.2.first_fixed_surface_type.coordinate_name(),
                stat =
                    v.2.statistical_process
                        .clone()
                        .map(|s| s.abbv())
                        .unwrap_or("".to_string())
                        .to_string(),
                gen = v.2.generating_process.abbv(),
            );

            if hash_mapping.contains_key(&hash) {
                hash_mapping.get_mut(&hash).unwrap().push(k.clone());
            } else {
                hash_mapping.insert(hash.clone(), vec![k.clone()]);
            }

            if vars.contains_key(&var) {
                vars.get_mut(&var).unwrap().insert(hash);
            } else {
                let mut set = HashSet::new();
                set.insert(hash);
                vars.insert(var, set);
            }
        }

        let mut var_names = vec![];
        let mut var_mapping = HashMap::new();
        for (k, v) in vars.iter_mut() {
            if v.len() == 1 {
                var_names.push(k.to_lowercase());
                var_mapping.insert(
                    k.to_lowercase(),
                    v.iter()
                        .flat_map(|h| hash_mapping.get(h).unwrap().clone())
                        .collect::<Vec<String>>(),
                );
            } else {
                for hash in v.iter() {
                    var_names.push(format!("{var}_{hash}", var = k.to_lowercase()));
                    var_mapping.insert(
                        format!("{var}_{hash}", var = k.to_lowercase()),
                        hash_mapping.get(hash).unwrap().clone(),
                    );
                }
            }
        }

        GribDataset {
            mapping,
            var_names,
            var_mapping,
        }
    }

    #[getter]
    fn get_attrs(&self) -> HashMap<&str, &str> {
        let mut map = HashMap::new();
        map.insert("meta", "Generated with gribberishpy");
        map
    }

    #[getter]
    fn get_vars(&self) -> Vec<String> {
        self.var_names.clone()
    }

    #[getter]
    fn get_coords<'py>(&self, py: Python<'py>) -> &'py PyDict {
        let first = self.mapping.values().next().unwrap();

        let coords = PyDict::new(py);

        // First we handle the spatial coords, which we assume to be shareed for all variables
        let latitude = PyDict::new(py);
        let latitude_metadata = PyDict::new(py);
        latitude_metadata
            .set_item("standard_name", "latitude")
            .unwrap();
        latitude_metadata.set_item("long_name", "latitude").unwrap();
        latitude_metadata.set_item("unit", "degrees_north").unwrap();
        latitude_metadata.set_item("axis", "Y").unwrap();
        latitude.set_item("attrs", latitude_metadata).unwrap();
        latitude
            .set_item("values", PyArray::from_slice(py, &first.2.latitude))
            .unwrap();

        let longitude = PyDict::new(py);
        let longitude_metadata = PyDict::new(py);
        longitude_metadata
            .set_item("standard_name", "longitude")
            .unwrap();
        longitude_metadata
            .set_item("long_name", "longitude")
            .unwrap();
        longitude_metadata.set_item("unit", "degrees_east").unwrap();
        longitude_metadata.set_item("axis", "X").unwrap();
        longitude.set_item("attrs", longitude_metadata).unwrap();

        longitude
            .set_item("values", PyArray::from_slice(py, &first.2.longitude))
            .unwrap();

        if first.2.is_regular_grid {
            latitude.set_item("dims", vec!["latitude"]).unwrap();
            longitude.set_item("dims", vec!["longitude"]).unwrap();
        } else {
            latitude.set_item("dims", vec!["y", "x"]).unwrap();
            longitude.set_item("dims", vec!["y", "x"]).unwrap();
        }

        coords.set_item("latitude", latitude).unwrap();
        coords.set_item("longitude", longitude).unwrap();

        // Temporal dims
        let mut time_map = HashMap::new();
        for (_var, v) in self.var_mapping.iter() {
            let mut times = HashSet::new();
            for k in v.iter() {
                times.insert(self.mapping.get(k).unwrap().2.forecast_date.clone());
            }
            let mut times = times.into_iter().collect::<Vec<_>>();
            times.sort();
            let time_key: String = times
                .iter()
                .map(|d| d.timestamp().to_string())
                .collect::<Vec<_>>()
                .join("_");

            time_map.insert(time_key, times);
        }

        let mut time_index = 0;
        for (_var, times) in time_map.iter() {
            let name = if time_map.len() == 1 || time_index == 0 {
                "time".to_string()
            } else {
                format!("time_{time_index}")
            };
            time_index += 1;

            let times = times
                .iter()
                .map(|d| PyDateTime::from_timestamp(py, d.timestamp() as f64, None).unwrap())
                .collect::<Vec<_>>();

            let time = PyDict::new(py);
            let time_metadata = PyDict::new(py);
            time_metadata.set_item("standard_name", "time").unwrap();
            time_metadata.set_item("long_name", "time").unwrap();
            time_metadata
                .set_item("unit", "seconds since 1970-01-01 00:00:00")
                .unwrap();
            time_metadata.set_item("axis", "T").unwrap();
            time.set_item("attrs", time_metadata).unwrap();
            time.set_item("values", times).unwrap();
            time.set_item("dims", vec!["time"]).unwrap();
            coords.set_item(name, time).unwrap();
        }

        // Vertical dims
        let mut vertical_map = HashMap::new();
        for (_var, v) in self.var_mapping.iter() {
            let mut verticals = HashSet::new();
            let mut vertical_name = String::new();
            for k in v.iter() {
                vertical_name = self
                    .mapping
                    .get(k)
                    .unwrap()
                    .2
                    .first_fixed_surface_type
                    .coordinate_name()
                    .to_string();
                if let Some(vertical_value) =
                    self.mapping.get(k).unwrap().2.first_fixed_surface_value
                {
                    verticals.insert(format!("{:.5}", vertical_value));
                }
            }

            if verticals.len() < 2 {
                continue;
            }

            let mut verticals = verticals
                .into_iter()
                .map(|f| f.parse::<f64>().unwrap())
                .collect::<Vec<_>>();
            verticals.sort_by(|a, b| a.partial_cmp(b).unwrap());

            vertical_map.insert(vertical_name, verticals);
        }

        for (var, values) in vertical_map.iter() {
            let vertical = PyDict::new(py);
            let vertical_metadata = PyDict::new(py);
            vertical_metadata.set_item("standard_name", var).unwrap();
            vertical_metadata.set_item("long_name", var).unwrap();
            // vertical_metadata
            //     .set_item("unit", "meters")
            //     .unwrap();
            vertical_metadata.set_item("axis", "Z").unwrap();
            vertical.set_item("attrs", vertical_metadata).unwrap();
            vertical.set_item("values", values).unwrap();
            vertical.set_item("dims", vec![var]).unwrap();
            coords.set_item(var, vertical).unwrap();
        }

        coords
    }

    // fn times<'py>(&self, py: Python<'py>) -> Vec<&'py PyDateTime> {
    //     let mut times = HashSet::new();
    //     for (_, v) in self.mapping.iter() {
    //         times.insert(v.2.forecast_date.clone());
    //     }
    //     times
    //         .into_iter()
    //         .map(|d| PyDateTime::from_timestamp(py, d.timestamp() as f64, None).unwrap())
    //         .collect::<Vec<_>>()
    // }
}
