use std::collections::{HashMap, HashSet};

use gribberish::message_metadata::{scan_message_metadata, MessageMetadata};
use pyo3::{prelude::*, types::PyList};

#[pyclass]
pub struct GribDataArray {
    offset: usize,
    metadata: MessageMetadata,
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

        let mapping = scan_message_metadata(data)
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
}
