use std::collections::{HashMap, HashSet};

use gribberish::{
    message::Message, message_metadata::scan_message_metadata,
    templates::product::tables::FixedSurfaceType,
};
use numpy::{
    datetime::{units::Seconds, Datetime},
    ndarray::{Dim, IxDynImpl},
    PyArray, PyArray1,
};
use pyo3::{
    prelude::*,
    types::{PyDict, PyList}, exceptions::PyValueError,
};

#[pyfunction]
pub fn build_grib_array<'py>(
    py: Python<'py>,
    data: &[u8],
    shape: Vec<usize>,
    offsets: Vec<usize>,
) -> &'py PyArray<f64, Dim<IxDynImpl>> {
    // Every grib chunk is going to be assumed to be the size of one entire message spatially
    let chunk_size = shape.iter().rev().take(2).product::<usize>();

    let v = offsets
        .iter()
        .flat_map(|offset| {
            let message = Message::from_data(data, *offset).unwrap();
            // Should this be resized?
            let mut data = message.data().unwrap();
            data.resize(chunk_size, 0.0);
            data
        })
        .collect::<Vec<_>>();

    let v = PyArray::from_slice(py, &v);
    // let full_size = shape.iter().product::<usize>();
    // if v.len() != full_size {
    //     unsafe {
    //         v.resize([full_size]).unwrap();
    //     }
    // }

    v.reshape(shape).unwrap()
}

#[pyfunction]
pub fn parse_grid_dataset<'py>(
    py: Python<'py>,
    data: &[u8],
    drop_variables: Option<&PyList>,
    only_variables: Option<&PyList>,
    perserve_dims: Option<&PyList>,
) -> PyResult<&'py PyDict> {
    let drop_variables = if let Some(drop_variables) = drop_variables {
        drop_variables
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let only_variables = if let Some(only_variables) = only_variables {
        Some(
            only_variables
                .iter()
                .map(|d| d.to_string().to_lowercase())
                .collect::<Vec<String>>(),
        )
    } else {
        None
    };

    let perserve_dims: Vec<String> = if let Some(perserve_dims) = perserve_dims {
        perserve_dims
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let mapping = scan_message_metadata(data)
        .into_iter()
        .filter_map(|(k, v)| {
            if &v.2.name.to_lowercase() == "missing" {
                None
            } else {
                Some((k.clone(), v))
            }
        })
        .collect::<HashMap<_, _>>();

    if mapping.keys().len() == 0 {
        return Err(PyValueError::new_err(
            "No valid GRIB messages found",
        ));
    }

    let mut vars: HashMap<String, HashSet<String>> = HashMap::new();
    let mut hash_mapping: HashMap<String, Vec<String>> = HashMap::new();

    for (k, v) in mapping.iter() {
        let var = v.2.var.clone();
        let hash = format!(
            "{var}{surf}_{stat}{gen}",
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
            let var_name = k.to_lowercase();
            if only_variables.is_some() && !only_variables.as_ref().unwrap().contains(&var_name) {
                continue;
            } else if drop_variables.contains(&var_name) {
                continue;
            }

            var_names.push(var_name);
            var_mapping.insert(
                k.to_lowercase(),
                v.iter()
                    .flat_map(|h| hash_mapping.get(h).unwrap().clone())
                    .collect::<Vec<String>>(),
            );
        } else {
            for hash in v.iter() {
                let var_name = format!("{var}_{hash}", var = k.to_lowercase());
                if only_variables.is_some() && !only_variables.as_ref().unwrap().contains(&var_name)
                {
                    continue;
                } else if drop_variables.contains(&var_name) {
                    continue;
                }

                var_names.push(var_name);
                var_mapping.insert(
                    format!("{var}_{hash}", var = k.to_lowercase()),
                    hash_mapping.get(hash).unwrap().clone(),
                );
            }
        }
    }

    let mut var_dims = var_mapping
        .keys()
        .map(|d| (d.to_owned(), vec![]))
        .collect::<HashMap<String, Vec<String>>>();
    let mut var_shape = var_mapping
        .keys()
        .map(|d| (d.to_owned(), vec![]))
        .collect::<HashMap<String, Vec<usize>>>();

    let coords = PyDict::new(py);

    // Temporal dims
    let mut time_map = HashMap::new();
    let mut time_dim_map: HashMap<String, Vec<String>> = HashMap::new();
    for (var, v) in var_mapping.iter() {
        let mut times = HashSet::new();
        for k in v.iter() {
            times.insert(mapping.get(k).unwrap().2.forecast_date.clone());
        }
        let mut times = times.into_iter().collect::<Vec<_>>();
        times.sort();
        let time_key: String = times
            .iter()
            .map(|d| d.timestamp().to_string())
            .collect::<Vec<_>>()
            .join("_");

        if time_dim_map.contains_key(&time_key) {
            time_dim_map.get_mut(&time_key).unwrap().push(var.clone());
        } else {
            time_dim_map.insert(time_key.clone(), vec![var.clone()]);
        }
        time_map.insert(time_key, times);
    }

    let mut time_index = 0;
    for (var, times) in time_map.iter() {
        let name = if time_map.len() == 1 || time_index == 0 {
            "time".to_string()
        } else {
            format!("time_{time_index}")
        };
        time_index += 1;

        let times = times
            .iter()
            .map(|d| Datetime::<Seconds>::from(d.timestamp()))
            .collect::<Vec<_>>();
        let times = PyArray1::from_slice(py, &times);

        time_dim_map[var].iter().for_each(|v: &String| {
            var_dims.get_mut(v).unwrap().push(name.clone());
            var_shape.get_mut(v).unwrap().push(times.len());
        });

        let time = PyDict::new(py);
        let time_metadata = PyDict::new(py);
        time_metadata.set_item("standard_name", "time").unwrap();
        time_metadata.set_item("long_name", "time").unwrap();
        time_metadata
            .set_item("unit", "seconds since 1970-01-01 00:00:00")
            .unwrap();
        time_metadata.set_item("axis", "T").unwrap();
        time.set_item("values", times).unwrap();
        time.set_item("attrs", time_metadata).unwrap();
        time.set_item("dims", vec![name.clone()]).unwrap();
        coords.set_item(name, time).unwrap();
    }

    // Vertical dims
    let mut vertical_map = HashMap::new();
    let mut vertical_dim_name_map: HashMap<String, HashSet<String>> = HashMap::new();
    let mut vertical_dim_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut vertical_attr_map: HashMap<String, FixedSurfaceType> = HashMap::new();
    for (var, v) in var_mapping.iter() {
        let mut verticals = HashSet::new();
        let mut vertical_name = String::new();
        for k in v.iter() {
            if vertical_name.is_empty() {
                vertical_name = mapping
                    .get(k)
                    .unwrap()
                    .2
                    .first_fixed_surface_type
                    .coordinate_name()
                    .to_string();
            }
            if let Some(vertical_value) = mapping.get(k).unwrap().2.first_fixed_surface_value {
                verticals.insert(format!("{:.5}", vertical_value));
            }
        }

        if !perserve_dims.contains(&vertical_name.to_lowercase()) && verticals.len() < 2 {
            continue;
        }

        let mut verticals = verticals
            .into_iter()
            .map(|f| f.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        verticals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let vertical_steps_key = verticals
            .iter()
            .map(|d| format!("{:.5}", d))
            .collect::<Vec<_>>()
            .join("_");
        let vertical_key = format!("{name}_{vertical_steps_key}", name = vertical_name);

        if vertical_dim_map.contains_key(&vertical_key) {
            vertical_dim_map
                .get_mut(&vertical_key)
                .unwrap()
                .push(var.clone());
        } else {
            vertical_dim_map.insert(vertical_key.clone(), vec![var.clone()]);
        }

        if vertical_dim_name_map.contains_key(&vertical_name) {
            vertical_dim_name_map
                .get_mut(&vertical_name)
                .unwrap()
                .insert(vertical_key.clone());
        } else {
            let mut vertical_key_set = HashSet::new();
            vertical_key_set.insert(vertical_key.clone());
            vertical_dim_name_map.insert(vertical_name.clone(), vertical_key_set);
        }

        vertical_map.insert(vertical_key, verticals);
        vertical_attr_map.insert(
            vertical_name,
            mapping
                .get(v.first().unwrap())
                .unwrap()
                .2
                .first_fixed_surface_type
                .clone(),
        );
    }

    for (dim, vertical_dims) in vertical_dim_name_map.iter() {
        let surface_type = vertical_attr_map.get(dim).unwrap();
        if vertical_dims.len() == 1 {
            let vertical_key = vertical_dims.iter().next().unwrap();
            let verticals = vertical_map.get(vertical_key).unwrap();
            vertical_dim_map
                .get_mut(vertical_key)
                .unwrap()
                .iter()
                .for_each(|v: &String| {
                    var_dims.get_mut(v).unwrap().push(dim.clone());
                    var_shape.get_mut(v).unwrap().push(verticals.len());
                });

            let vertical = PyDict::new(py);
            let vertical_metadata = PyDict::new(py);
            vertical_metadata
                .set_item("standard_name", surface_type.to_string())
                .unwrap();
            vertical_metadata
                .set_item("long_name", surface_type.to_string())
                .unwrap();
            // vertical_metadata
            //     .set_item("unit", "meters")
            //     .unwrap();
            if surface_type.is_vertical_level() {
                vertical_metadata.set_item("axis", "Z").unwrap();
            }
            vertical.set_item("attrs", vertical_metadata).unwrap();
            vertical.set_item("values", verticals).unwrap();
            vertical.set_item("dims", vec![dim]).unwrap();
            coords.set_item(dim, vertical).unwrap();
        } else {
            for (i, vertical_key) in vertical_dims.iter().enumerate() {
                let name = if vertical_dims.len() == 1 {
                    dim.clone()
                } else {
                    format!("{}_{}", dim, i)
                };

                let verticals = vertical_map.get(vertical_key).unwrap();
                vertical_dim_map
                    .get_mut(vertical_key)
                    .unwrap()
                    .iter()
                    .for_each(|v: &String| {
                        var_dims.get_mut(v).unwrap().push(name.clone());
                        var_shape.get_mut(v).unwrap().push(verticals.len());
                    });

                let vertical = PyDict::new(py);
                let vertical_metadata = PyDict::new(py);
                vertical_metadata
                    .set_item("standard_name", surface_type.to_string())
                    .unwrap();
                vertical_metadata
                    .set_item("long_name", surface_type.to_string())
                    .unwrap();
                // vertical_metadata
                //     .set_item("unit", "meters")
                //     .unwrap();
                if surface_type.is_vertical_level() {
                    vertical_metadata.set_item("axis", "Z").unwrap();
                }
                vertical.set_item("attrs", vertical_metadata).unwrap();
                vertical.set_item("values", verticals).unwrap();
                vertical.set_item("dims", vec![name.clone()]).unwrap();
                coords.set_item(name, vertical).unwrap();
            }
        }
    }

    // Lastly the spatial coords
    let latitude = PyDict::new(py);
    let latitude_metadata = PyDict::new(py);
    latitude_metadata
        .set_item("standard_name", "latitude")
        .unwrap();
    latitude_metadata.set_item("long_name", "latitude").unwrap();
    latitude_metadata.set_item("unit", "degrees_north").unwrap();
    latitude.set_item("attrs", latitude_metadata).unwrap();

    let longitude = PyDict::new(py);
    let longitude_metadata = PyDict::new(py);
    longitude_metadata
        .set_item("standard_name", "longitude")
        .unwrap();
    longitude_metadata
        .set_item("long_name", "longitude")
        .unwrap();
    longitude_metadata.set_item("unit", "degrees_east").unwrap();
    longitude.set_item("attrs", longitude_metadata).unwrap();

    let first = mapping.values().next().unwrap();
    let grid_shape = first.2.grid_shape;
    var_shape.iter_mut().for_each(|(_, v)| {
        v.push((&grid_shape).0);
        v.push((&grid_shape).1);
    });

    if first.2.is_regular_grid {
        latitude.set_item("dims", vec!["latitude"]).unwrap();
        latitude_metadata.set_item("axis", "Y").unwrap();

        let (lat, lng) = first.2.latlng();
        latitude
            .set_item("values", PyArray1::from_slice(py, &lat))
            .unwrap();

        longitude.set_item("dims", vec!["longitude"]).unwrap();
        longitude_metadata.set_item("axis", "X").unwrap();
        longitude
            .set_item("values", PyArray1::from_slice(py, &lng))
            .unwrap();

        var_dims.iter_mut().for_each(|(_, v)| {
            v.push("latitude".to_string());
            v.push("longitude".to_string());
        });
    } else {
        let y: &PyDict = PyDict::new(py);
        let y_metadata = PyDict::new(py);
        y_metadata.set_item("axis", "Y").unwrap();
        y_metadata
            .set_item("standard_name", "projection_y_coordinate")
            .unwrap();
        y_metadata
            .set_item("long_name", "y coordinate of projection")
            .unwrap();
        y_metadata.set_item("unit", "m").unwrap();
        y.set_item("attrs", y_metadata).unwrap();
        y.set_item("dims", vec!["y"]).unwrap();
        y.set_item("values", PyArray::from_slice(py, &first.2.projector.y()))
            .unwrap();
        coords.set_item("y", y).unwrap();

        let x = PyDict::new(py);
        let x_metadata = PyDict::new(py);
        x_metadata.set_item("axis", "X").unwrap();
        x_metadata
            .set_item("standard_name", "projection_x_coordinate")
            .unwrap();
        x_metadata
            .set_item("long_name", "x coordinate of projection")
            .unwrap();
        x_metadata.set_item("unit", "m").unwrap();
        x.set_item("attrs", x_metadata).unwrap();
        x.set_item("dims", vec!["x"]).unwrap();
        x.set_item("values", PyArray::from_slice(py, &first.2.projector.x()))
            .unwrap();
        coords.set_item("x", x).unwrap();

        latitude.set_item("dims", vec!["y", "x"]).unwrap();
        let (lat, lng) = first.2.latlng();
        let lats = PyArray::from_slice(py, &lat);
        let lats = lats.reshape([grid_shape.0, grid_shape.1]).unwrap();
        latitude.set_item("values", lats).unwrap();

        longitude.set_item("dims", vec!["y", "x"]).unwrap();
        let lngs = PyArray::from_slice(py, &lng);
        let lngs = lngs.reshape([grid_shape.0, grid_shape.1]).unwrap();
        longitude.set_item("values", lngs).unwrap();

        var_dims.iter_mut().for_each(|(_, v)| {
            v.push("y".to_string());
            v.push("x".to_string());
        });
    }

    coords.set_item("latitude", latitude).unwrap();
    coords.set_item("longitude", longitude).unwrap();

    // Vars
    let data_vars = PyDict::new(py);
    for (var, v) in var_mapping.iter() {
        let dims = var_dims.get(var).unwrap().clone();
        let shape = var_shape.get(var).unwrap().clone();
        let data_var = PyDict::new(py);
        let var_metadata = PyDict::new(py);
        let first = mapping.get(v.first().unwrap()).unwrap();
        var_metadata
            .set_item("standard_name", first.2.name.clone())
            .unwrap();
        var_metadata
            .set_item("long_name", first.2.name.clone())
            .unwrap();
        var_metadata
            .set_item("unit", first.2.units.clone())
            .unwrap();
        var_metadata
            .set_item("coordinates", "latitude longitude")
            .unwrap();
        var_metadata
            .set_item("reference_date", first.2.reference_date.to_rfc3339())
            .unwrap();
        var_metadata
            .set_item("generating_process", first.2.generating_process.to_string())
            .unwrap();
        if let Some(statistical_process) = first.2.statistical_process.as_ref() {
            var_metadata
                .set_item("statistical_process", statistical_process.to_string())
                .unwrap();
        }
        var_metadata.set_item("crs", first.2.proj.clone()).unwrap();

        data_var.set_item("attrs", var_metadata).unwrap();
        data_var.set_item("dims", dims).unwrap();

        let mut v_sorted = v.clone();
        v_sorted.sort_by(|a, b| {
            let a = mapping.get(a).unwrap();
            let b = mapping.get(b).unwrap();
            (
                a.2.forecast_date,
                a.2.first_fixed_surface_value.unwrap_or(0.0),
            )
                .partial_cmp(&(
                    b.2.forecast_date,
                    b.2.first_fixed_surface_value.unwrap_or(0.0),
                ))
                .unwrap()
        });

        let offsets = v_sorted
            .iter()
            .map(|chunk| {
                (
                    mapping.get(chunk).unwrap().1,
                    mapping.get(chunk).unwrap().2.message_size,
                )
            })
            .collect::<Vec<_>>();

        let array = PyDict::new(py);
        array.set_item("shape", shape).unwrap();
        array.set_item("offsets", offsets).unwrap();

        data_var.set_item("values", array).unwrap();
        data_vars.set_item(var, data_var).unwrap();
    }

    // Attrs
    let attrs = PyDict::new(py);
    attrs
        .set_item("meta", "Generated with gribberishpy")
        .unwrap();

    // Dataset
    let dataset = PyDict::new(py);
    dataset.set_item("coords", coords).unwrap();
    dataset.set_item("data_vars", data_vars).unwrap();
    dataset.set_item("attrs", attrs).unwrap();
    Ok(dataset)
}
