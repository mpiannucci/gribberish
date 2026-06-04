use std::collections::{BTreeMap, HashMap, HashSet};

use gribberish::{
    message_metadata::{scan_message_metadata, MessageMetadata},
    templates::product::tables::{FixedSurfaceType, ProbabilityType},
};
use numpy::{
    datetime::{units::Seconds, Datetime},
    PyArray, PyArray1, PyArrayMethods,
};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyList},
};

/// Compute the "kind" discriminator for a message: the statistical process,
/// ensemble product, probability descriptor and anomaly flag combined into a
/// single readable token. Plain instantaneous forecasts collapse to "instant".
/// This is what (potentially) separates two hypercubes of the same variable at
/// the same level type into different groups.
fn message_kind(meta: &MessageMetadata) -> String {
    let mut tokens: Vec<String> = Vec::new();

    if let Some(stat) = meta.statistical_process.as_ref() {
        let mut token = stat.abbv();
        // Distinguish accumulation/averaging windows (e.g. 1h vs 6h precip).
        if let Some(end_date) = meta.forecast_end_date {
            let hours = end_date
                .signed_duration_since(meta.forecast_date)
                .num_hours();
            if hours > 0 {
                token = format!("{token}{hours}h");
            }
        }
        tokens.push(token);
    }

    if let Some(derived) = meta.derived_forecast_type.as_ref() {
        tokens.push(derived.abbv());
    }

    if let Some(prob) = meta.probability_type.as_ref() {
        let mut token = prob.abbv();
        if matches!(
            prob,
            ProbabilityType::BetweenLimits | ProbabilityType::BetweenLimitsInclusive
        ) {
            if let (Some(lower), Some(upper)) =
                (meta.probability_lower_limit, meta.probability_upper_limit)
            {
                token = format!("{token}_{lower:.0}_{upper:.0}");
            }
        }
        tokens.push(token);
    } else if meta.percentile_value.is_some() {
        // Percentile products share the "pctl" kind so they never collapse into
        // the same array as raw members; the value itself stays a dimension.
        tokens.push("pctl".to_string());
    }

    if meta.is_anomaly {
        tokens.push("anom".to_string());
    }

    if tokens.is_empty() {
        "instant".to_string()
    } else {
        tokens.join("_")
    }
}

#[pyfunction]
#[pyo3(signature = (data, drop_variables=None, only_variables=None, perserve_dims=None, filter_by_attrs=None, filter_by_variable_attrs=None, encode_coords=None))]
#[allow(clippy::too_many_arguments)]
pub fn parse_grib_dataset<'py>(
    py: Python<'py>,
    data: &[u8],
    drop_variables: Option<&Bound<PyList>>,
    only_variables: Option<&Bound<PyList>>,
    perserve_dims: Option<&Bound<PyList>>,
    filter_by_attrs: Option<&Bound<'py, PyDict>>,
    filter_by_variable_attrs: Option<&Bound<'py, PyDict>>,
    encode_coords: Option<bool>,
) -> PyResult<Bound<'py, PyDict>> {
    let drop_variables = if let Some(drop_variables) = drop_variables {
        drop_variables
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let only_variables = only_variables.map(|only_variables| {
        only_variables
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect::<Vec<String>>()
    });

    let perserve_dims: Vec<String> = if let Some(perserve_dims) = perserve_dims {
        perserve_dims
            .iter()
            .map(|d| d.to_string().to_lowercase())
            .collect::<Vec<String>>()
    } else {
        Vec::new()
    };

    let filter_by_attrs = if let Some(filter_by_attrs) = filter_by_attrs {
        filter_by_attrs.clone()
    } else {
        PyDict::new(py)
    };

    let (filter_by_variable_attrs, filter_by_variable_attrs_defined) =
        if let Some(filter_by_variable_attrs) = filter_by_variable_attrs {
            (filter_by_variable_attrs.clone(), true)
        } else {
            (PyDict::new(py), false)
        };

    let encode_coords = encode_coords.unwrap_or_default();

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
        return Err(PyValueError::new_err("No valid GRIB messages found"));
    }

    // Determine how to split the messages into groups, mirroring how cfgrib
    // breaks a file into multiple datasets. Two discriminators can force a split:
    //   * level type ("sfc", "isobar", ...) -> top-level group
    //   * "kind" (statistical process / ensemble product / probability / anomaly)
    //       -> nested group
    // A discriminator only becomes a group axis when at least one variable
    // actually spans more than one of its values; otherwise the corresponding
    // coordinate stays a dimension and everything lands in a single root dataset.
    let mut var_levels: HashMap<String, HashSet<String>> = HashMap::new();
    let mut var_level_kinds: HashMap<(String, String), HashSet<String>> = HashMap::new();
    let mut msg_info: HashMap<String, (String, String, String)> = HashMap::new();
    for (k, v) in mapping.iter() {
        let meta = &v.2;
        let var = meta.var.to_lowercase();
        let level = meta.first_fixed_surface_type.coordinate_name().to_string();
        let kind = message_kind(meta);
        var_levels
            .entry(var.clone())
            .or_default()
            .insert(level.clone());
        var_level_kinds
            .entry((var.clone(), level.clone()))
            .or_default()
            .insert(kind.clone());
        msg_info.insert(k.clone(), (var, level, kind));
    }

    let partition_by_level = var_levels.values().any(|levels| levels.len() > 1);
    let partition_by_kind = var_level_kinds.values().any(|kinds| kinds.len() > 1);

    // group path (0, 1 or 2 segments) -> variable short name -> message keys
    let mut groups: BTreeMap<Vec<String>, HashMap<String, Vec<String>>> = BTreeMap::new();
    for (k, (var, level, kind)) in msg_info.iter() {
        if (only_variables.is_some() && !only_variables.as_ref().unwrap().contains(var))
            || drop_variables.contains(var)
        {
            continue;
        }

        let mut path = Vec::new();
        if partition_by_level {
            path.push(level.clone());
        }
        if partition_by_kind {
            path.push(kind.clone());
        }

        groups
            .entry(path)
            .or_default()
            .entry(var.clone())
            .or_default()
            .push(k.clone());
    }

    if groups.is_empty() {
        return Err(PyValueError::new_err("No variables remain after filtering"));
    }

    // No conflicts: a single root dataset, with levels expressed as dimensions.
    if groups.len() == 1 && groups.keys().next().unwrap().is_empty() {
        let (_, var_mapping) = groups.into_iter().next().unwrap();
        return build_group(
            py,
            &mapping,
            &var_mapping,
            &perserve_dims,
            &filter_by_attrs,
            &filter_by_variable_attrs,
            filter_by_variable_attrs_defined,
            encode_coords,
        );
    }

    // Conflicts: build a tree of standalone group datasets.
    let root = PyDict::new(py);
    root.set_item("coords", PyDict::new(py))?;
    root.set_item("data_vars", PyDict::new(py))?;
    let root_attrs = PyDict::new(py);
    root_attrs.set_item("meta", "Generated with gribberishpy")?;
    root.set_item("attrs", root_attrs)?;

    let groups_dict = PyDict::new(py);
    // first path segment -> the "groups" sub-dict of its intermediate node
    let mut intermediates: HashMap<String, Bound<'py, PyDict>> = HashMap::new();
    for (path, var_mapping) in groups.iter() {
        let node = build_group(
            py,
            &mapping,
            var_mapping,
            &perserve_dims,
            &filter_by_attrs,
            &filter_by_variable_attrs,
            filter_by_variable_attrs_defined,
            encode_coords,
        )?;

        match path.as_slice() {
            [segment] => {
                groups_dict.set_item(segment, node)?;
            }
            [parent, child] => {
                let sub = if let Some(sub) = intermediates.get(parent) {
                    sub.clone()
                } else {
                    let intermediate = PyDict::new(py);
                    intermediate.set_item("coords", PyDict::new(py))?;
                    intermediate.set_item("data_vars", PyDict::new(py))?;
                    let attrs = PyDict::new(py);
                    attrs.set_item("meta", "Generated with gribberishpy")?;
                    intermediate.set_item("attrs", attrs)?;
                    let sub = PyDict::new(py);
                    intermediate.set_item("groups", &sub)?;
                    groups_dict.set_item(parent, intermediate)?;
                    intermediates.insert(parent.clone(), sub.clone());
                    sub
                };
                sub.set_item(child, node)?;
            }
            _ => {}
        }
    }
    root.set_item("groups", groups_dict)?;

    Ok(root)
}

/// Build a single, standalone group dataset (coords + data_vars + attrs) from a
/// mapping of variable short names to the GRIB message keys that compose them.
#[allow(clippy::too_many_arguments)]
fn build_group<'py>(
    py: Python<'py>,
    mapping: &HashMap<String, (usize, usize, MessageMetadata)>,
    var_mapping: &HashMap<String, Vec<String>>,
    perserve_dims: &[String],
    filter_by_attrs: &Bound<'py, PyDict>,
    filter_by_variable_attrs: &Bound<'py, PyDict>,
    filter_by_variable_attrs_defined: bool,
    encode_coords: bool,
) -> PyResult<Bound<'py, PyDict>> {
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
            if let Some(forecast_end_date) = mapping.get(k).unwrap().2.forecast_end_date {
                times.insert(forecast_end_date);
            } else {
                times.insert(mapping.get(k).unwrap().2.forecast_date);
            }
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

    for (time_index, (var, times)) in time_map.iter().enumerate() {
        let name = if time_map.len() == 1 || time_index == 0 {
            "time".to_string()
        } else {
            format!("time_{time_index}")
        };

        let times = times
            .iter()
            .map(|d| Datetime::<Seconds>::from(d.timestamp()))
            .collect::<Vec<_>>();
        let times = PyArray1::from_vec(py, times);

        time_dim_map[var].iter().for_each(|v: &String| {
            var_dims.get_mut(v).unwrap().push(name.clone());
            var_shape.get_mut(v).unwrap().push(times.len().unwrap());
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

    // Ensemble member dims
    let mut member_map = HashMap::new();
    let mut member_dim_map: HashMap<String, Vec<String>> = HashMap::new();
    for (var, v) in var_mapping.iter() {
        let mut members = HashSet::new();
        for k in v.iter() {
            if let Some(perturbation_number) = mapping.get(k).unwrap().2.perturbation_number {
                members.insert(perturbation_number);
            }
        }

        if members.is_empty() {
            continue;
        }

        let mut members = members.into_iter().collect::<Vec<_>>();
        members.sort();

        let member_key: String = members
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join("_");

        if member_dim_map.contains_key(&member_key) {
            member_dim_map
                .get_mut(&member_key)
                .unwrap()
                .push(var.clone());
        } else {
            member_dim_map.insert(member_key.clone(), vec![var.clone()]);
        }
        member_map.insert(member_key, members);
    }

    for (member_index, (member_key, members)) in member_map.iter().enumerate() {
        let name = if member_map.len() == 1 || member_index == 0 {
            "number".to_string()
        } else {
            format!("number_{member_index}")
        };

        let members_i64 = members.iter().map(|m| *m as i64).collect::<Vec<_>>();
        let members_array = PyArray1::from_vec(py, members_i64);

        member_dim_map[member_key].iter().for_each(|v: &String| {
            var_dims.get_mut(v).unwrap().push(name.clone());
            var_shape.get_mut(v).unwrap().push(members.len());
        });

        let member = PyDict::new(py);
        let member_metadata = PyDict::new(py);
        member_metadata
            .set_item("standard_name", "realization")
            .unwrap();
        member_metadata
            .set_item("long_name", "ensemble member")
            .unwrap();
        member_metadata.set_item("axis", "E").unwrap();
        member.set_item("values", members_array).unwrap();
        member.set_item("attrs", member_metadata).unwrap();
        member.set_item("dims", vec![name.clone()]).unwrap();
        coords.set_item(name, member).unwrap();
    }

    // Percentile dims
    let mut percentile_map = HashMap::new();
    let mut percentile_dim_map: HashMap<String, Vec<String>> = HashMap::new();
    for (var, v) in var_mapping.iter() {
        let mut percentiles = HashSet::new();
        for k in v.iter() {
            if let Some(percentile_value) = mapping.get(k).unwrap().2.percentile_value {
                percentiles.insert(percentile_value);
            }
        }

        if !perserve_dims.contains(&"percentile".to_string()) && percentiles.len() < 2 {
            continue;
        }

        let mut percentiles = percentiles.into_iter().collect::<Vec<_>>();
        percentiles.sort();

        let percentile_key: String = percentiles
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join("_");

        if percentile_dim_map.contains_key(&percentile_key) {
            percentile_dim_map
                .get_mut(&percentile_key)
                .unwrap()
                .push(var.clone());
        } else {
            percentile_dim_map.insert(percentile_key.clone(), vec![var.clone()]);
        }
        percentile_map.insert(percentile_key, percentiles);
    }

    for (percentile_index, (percentile_key, percentiles)) in percentile_map.iter().enumerate() {
        let name = if percentile_map.len() == 1 || percentile_index == 0 {
            "percentile".to_string()
        } else {
            format!("percentile_{percentile_index}")
        };

        let percentiles_i64 = percentiles.iter().map(|p| *p as i64).collect::<Vec<_>>();
        let percentiles_array = PyArray1::from_vec(py, percentiles_i64);

        percentile_dim_map[percentile_key]
            .iter()
            .for_each(|v: &String| {
                var_dims.get_mut(v).unwrap().push(name.clone());
                var_shape.get_mut(v).unwrap().push(percentiles.len());
            });

        let percentile = PyDict::new(py);
        let percentile_metadata = PyDict::new(py);
        percentile_metadata
            .set_item("standard_name", "percentile")
            .unwrap();
        percentile_metadata
            .set_item("long_name", "percentile")
            .unwrap();
        percentile_metadata.set_item("unit", "%").unwrap();
        percentile.set_item("values", percentiles_array).unwrap();
        percentile.set_item("attrs", percentile_metadata).unwrap();
        percentile.set_item("dims", vec![name.clone()]).unwrap();
        coords.set_item(name, percentile).unwrap();
    }

    // Threshold dims (for probability variables with varying limits)
    let mut threshold_map: HashMap<String, Vec<f64>> = HashMap::new();
    let mut threshold_dim_map: HashMap<String, Vec<String>> = HashMap::new();
    for (var, v) in var_mapping.iter() {
        let mut thresholds = HashSet::new();
        let mut has_probability = false;
        for k in v.iter() {
            let meta = &mapping.get(k).unwrap().2;
            if meta.probability_type.is_some() {
                has_probability = true;
                // Only create a threshold dimension for single-limit types
                // (e.g., P(X < threshold)). Between-type probabilities are
                // already split into separate variables via the hash.
                let threshold = match &meta.probability_type {
                    Some(ProbabilityType::BetweenLimits)
                    | Some(ProbabilityType::BetweenLimitsInclusive) => None,
                    _ => meta
                        .probability_lower_limit
                        .or(meta.probability_upper_limit),
                };
                if let Some(t) = threshold {
                    thresholds.insert(format!("{:.5}", t));
                }
            }
        }

        if !has_probability || thresholds.len() < 2 {
            continue;
        }

        let mut thresholds = thresholds
            .into_iter()
            .map(|f| f.parse::<f64>().unwrap())
            .collect::<Vec<_>>();
        thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let threshold_key = thresholds
            .iter()
            .map(|t| format!("{:.5}", t))
            .collect::<Vec<_>>()
            .join("_");

        if threshold_dim_map.contains_key(&threshold_key) {
            threshold_dim_map
                .get_mut(&threshold_key)
                .unwrap()
                .push(var.clone());
        } else {
            threshold_dim_map.insert(threshold_key.clone(), vec![var.clone()]);
        }
        threshold_map.insert(threshold_key, thresholds);
    }

    for (threshold_index, (threshold_key, thresholds)) in threshold_map.iter().enumerate() {
        let name = if threshold_map.len() == 1 || threshold_index == 0 {
            "threshold".to_string()
        } else {
            format!("threshold_{threshold_index}")
        };

        let thresholds_array = PyArray1::from_vec(py, thresholds.clone());

        threshold_dim_map[threshold_key]
            .iter()
            .for_each(|v: &String| {
                var_dims.get_mut(v).unwrap().push(name.clone());
                var_shape.get_mut(v).unwrap().push(thresholds.len());
            });

        let threshold = PyDict::new(py);
        let threshold_metadata = PyDict::new(py);
        threshold_metadata
            .set_item("standard_name", "threshold")
            .unwrap();
        threshold_metadata
            .set_item("long_name", "probability threshold")
            .unwrap();
        threshold.set_item("values", thresholds_array).unwrap();
        threshold.set_item("attrs", threshold_metadata).unwrap();
        threshold.set_item("dims", vec![name.clone()]).unwrap();
        coords.set_item(name, threshold).unwrap();
    }

    // Lastly the spatial coords
    let latitude = PyDict::new(py);
    let latitude_metadata = PyDict::new(py);
    latitude_metadata
        .set_item("standard_name", "latitude")
        .unwrap();
    latitude_metadata.set_item("long_name", "latitude").unwrap();
    latitude_metadata.set_item("unit", "degrees_north").unwrap();
    latitude.set_item("attrs", &latitude_metadata).unwrap();

    let longitude = PyDict::new(py);
    let longitude_metadata = PyDict::new(py);
    longitude_metadata
        .set_item("standard_name", "longitude")
        .unwrap();
    longitude_metadata
        .set_item("long_name", "longitude")
        .unwrap();
    longitude_metadata.set_item("unit", "degrees_east").unwrap();
    longitude.set_item("attrs", &longitude_metadata).unwrap();

    let first = mapping
        .get(var_mapping.values().next().unwrap().first().unwrap())
        .unwrap();
    let grid_shape = first.2.grid_shape;
    var_shape.iter_mut().for_each(|(_, v)| {
        v.push(grid_shape.0);
        v.push(grid_shape.1);
    });

    if first.2.is_regular_grid {
        latitude.set_item("dims", vec!["latitude"]).unwrap();
        latitude_metadata.set_item("axis", "Y").unwrap();

        let (lat, lng) = first.2.latlng();
        latitude
            .set_item("values", PyArray1::from_vec(py, lat))
            .unwrap();

        longitude.set_item("dims", vec!["longitude"]).unwrap();
        longitude_metadata.set_item("axis", "X").unwrap();
        longitude
            .set_item("values", PyArray1::from_vec(py, lng))
            .unwrap();

        var_dims.iter_mut().for_each(|(_, v)| {
            v.push("latitude".to_string());
            v.push("longitude".to_string());
        });
    } else {
        let y = PyDict::new(py);
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
        y.set_item("values", PyArray::from_vec(py, first.2.projector.y()))
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
        x.set_item("values", PyArray::from_vec(py, first.2.projector.x()))
            .unwrap();
        coords.set_item("x", x).unwrap();

        latitude.set_item("dims", vec!["y", "x"]).unwrap();
        longitude.set_item("dims", vec!["y", "x"]).unwrap();

        if encode_coords {
            let lats_array = PyDict::new(py);
            lats_array
                .set_item("shape", [grid_shape.0, grid_shape.1])
                .unwrap();
            lats_array
                .set_item("offsets", [(first.1, first.2.message_size)])
                .unwrap();
            latitude.set_item("values", lats_array).unwrap();

            let lngs_array = PyDict::new(py);
            lngs_array
                .set_item("shape", [grid_shape.0, grid_shape.1])
                .unwrap();
            lngs_array
                .set_item("offsets", [(first.1, first.2.message_size)])
                .unwrap();
            longitude.set_item("values", lngs_array).unwrap();
        } else {
            let (lat, lng) = first.2.latlng();
            let lats = PyArray::from_vec(py, lat);
            let lats = lats.reshape([grid_shape.0, grid_shape.1]).unwrap();
            latitude.set_item("values", lats).unwrap();

            let lngs = PyArray::from_vec(py, lng);
            let lngs = lngs.reshape([grid_shape.0, grid_shape.1]).unwrap();
            longitude.set_item("values", lngs).unwrap();
        }

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
            .set_item("discipline", first.2.discipline_value as i32)
            .unwrap();
        var_metadata
            .set_item("parameterCategory", first.2.category_value as i32)
            .unwrap();
        var_metadata
            .set_item("parameterNumber", first.2.parameter_value as i32)
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
            .set_item("forecast_date", first.2.forecast_date.to_rfc3339())
            .unwrap();
        var_metadata
            .set_item(
                "forecast_end_date",
                first
                    .2
                    .forecast_end_date
                    .map_or("".to_string(), |d| d.to_rfc3339()),
            )
            .unwrap();
        var_metadata
            .set_item(
                "fixed_surface_type",
                first.2.first_fixed_surface_type.to_string(),
            )
            .unwrap();
        var_metadata
            .set_item(
                "fixed_surface_value",
                first
                    .2
                    .first_fixed_surface_value
                    .map_or("".to_string(), |f| f.to_string()),
            )
            .unwrap();
        var_metadata
            .set_item(
                "first_fixed_surface_type_coordinate",
                first.2.first_fixed_surface_type.coordinate_name(),
            )
            .unwrap();
        var_metadata
            .set_item("generating_process", first.2.generating_process.to_string())
            .unwrap();
        var_metadata
            .set_item(
                "statistical_process",
                first
                    .2
                    .statistical_process
                    .as_ref()
                    .map(|s| s.to_string())
                    .unwrap_or("".to_string()),
            )
            .unwrap();
        var_metadata
            .set_item(
                "probability_type",
                first
                    .2
                    .probability_type
                    .as_ref()
                    .map(|p| p.to_string())
                    .unwrap_or("".to_string()),
            )
            .unwrap();

        let proj_params = PyDict::new(py);
        proj_params
            .set_item("proj", first.2.projector.proj_name())
            .unwrap();
        first.2.projector.proj_params().iter().for_each(|(k, v)| {
            proj_params.set_item(k, v).unwrap();
        });
        var_metadata.set_item("proj_params", proj_params).unwrap();

        var_metadata.set_item("crs", first.2.proj.clone()).unwrap();

        let mut filtered = false;
        if filter_by_variable_attrs_defined {
            if let Ok(Some(filter_by_variable_attrs)) = filter_by_variable_attrs.get_item(var) {
                let filter_by_variable_attrs = filter_by_variable_attrs.cast::<PyDict>().unwrap();
                for attr in filter_by_variable_attrs.keys() {
                    if filter_by_variable_attrs.contains(&attr).unwrap() {
                        let filter_value = filter_by_variable_attrs
                            .get_item(&attr)
                            .unwrap()
                            .unwrap()
                            .to_string();
                        let var_value = var_metadata.get_item(attr).unwrap().unwrap().to_string();
                        if filter_value != var_value {
                            filtered = true;
                            break;
                        }
                    }
                }
            }
        } else {
            for attr in var_metadata.keys() {
                if filter_by_attrs.contains(&attr).unwrap() {
                    let filter_value = filter_by_attrs
                        .get_item(&attr)
                        .unwrap()
                        .unwrap()
                        .to_string();
                    let var_value = var_metadata.get_item(attr).unwrap().unwrap().to_string();
                    if filter_value != var_value {
                        filtered = true;
                        break;
                    }
                }
            }
        }

        if filtered {
            continue;
        }

        data_var.set_item("attrs", var_metadata).unwrap();
        data_var.set_item("dims", dims).unwrap();

        let mut v_sorted = v.clone();
        v_sorted.sort_by(|a, b| {
            let a = mapping.get(a).unwrap();
            let b = mapping.get(b).unwrap();
            let a_threshold =
                a.2.probability_lower_limit
                    .or(a.2.probability_upper_limit)
                    .unwrap_or(0.0);
            let b_threshold =
                b.2.probability_lower_limit
                    .or(b.2.probability_upper_limit)
                    .unwrap_or(0.0);
            (
                a.2.forecast_date,
                a.2.first_fixed_surface_value.unwrap_or(0.0),
                a.2.perturbation_number.unwrap_or(0),
                a.2.percentile_value.unwrap_or(0),
                format!("{:.5}", a_threshold),
            )
                .partial_cmp(&(
                    b.2.forecast_date,
                    b.2.first_fixed_surface_value.unwrap_or(0.0),
                    b.2.perturbation_number.unwrap_or(0),
                    b.2.percentile_value.unwrap_or(0),
                    format!("{:.5}", b_threshold),
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
