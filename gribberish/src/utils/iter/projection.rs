use std::collections::HashMap;

use itertools::Itertools;
use mappers::{projections::LambertConformalConic, Projection};

#[derive(Clone, Debug)]
pub struct PlateCareeProjection {
    pub latitudes: RegularCoordinateIterator,
    pub longitudes: RegularCoordinateIterator,
    pub projection_name: String,
    pub projection_params: HashMap<String, f64>,
}

#[derive(Clone, Debug)]
pub struct LambertConformalConicProjection {
    pub x: RegularCoordinateIterator,
    pub y: RegularCoordinateIterator,
    pub projection: LambertConformalConic,
    pub projection_name: String,
    pub projection_params: HashMap<String, f64>,
}

/// Projection for Gaussian grids (irregular latitudes, regular longitudes)
#[derive(Clone, Debug)]
pub struct GaussianProjection {
    pub latitudes: IrregularCoordinateIterator,
    pub longitudes: RegularCoordinateIterator,
    pub projection_name: String,
    pub projection_params: HashMap<String, f64>,
}

#[derive(Clone, Debug)]
pub enum LatLngProjection {
    PlateCaree(PlateCareeProjection),
    LambertConformal(LambertConformalConicProjection),
    Gaussian(GaussianProjection),
}

impl LatLngProjection {
    pub fn is_regular_latlng_grid(&self) -> bool {
        match self {
            LatLngProjection::PlateCaree(_) => true,
            LatLngProjection::LambertConformal(_) => false,
            LatLngProjection::Gaussian(_) => false,
        }
    }

    pub fn lat_lng(&self) -> (Vec<f64>, Vec<f64>) {
        match self {
            LatLngProjection::PlateCaree(projection) => {
                let lats: Vec<f64> = projection.latitudes.clone().collect();
                let lon_start = projection.longitudes.start;
                let lngs: Vec<f64> = projection.longitudes.clone()
                    .map(|lon| {
                        // Normalize to 0..360 range for grids that wrap around the globe
                        // (consistent with GRIB1 handling in grid_description.rs)
                        if lon >= 360.0 {
                            lon - 360.0
                        } else if lon < 0.0 && lon_start >= 0.0 {
                            lon + 360.0
                        } else {
                            lon
                        }
                    })
                    .collect();
                (lats, lngs)
            }
            LatLngProjection::LambertConformal(projection) => projection
                .y
                .clone()
                .flat_map(|y_coord| {
                    let mut x = projection.x.clone();
                    x.current_index = 0;
                    x.clone()
                        .map(|x_coord| {
                            let projected = projection
                                .projection
                                .inverse_project(x_coord, y_coord)
                                .expect("Failed to inverse project from xy to lnglat");
                            (projected.1, projected.0)
                        })
                        .collect::<Vec<(f64, f64)>>()
                })
                .unzip(),
            LatLngProjection::Gaussian(projection) => (
                projection.latitudes.clone().collect(),
                projection.longitudes.clone().collect(),
            ),
        }
    }

    pub fn x(&self) -> Vec<f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => {
                let lon_start = projection.longitudes.start;
                projection.longitudes.clone()
                    .map(|lon| {
                        // Normalize to 0..360 range for grids that wrap around the globe
                        if lon >= 360.0 {
                            lon - 360.0
                        } else if lon < 0.0 && lon_start >= 0.0 {
                            lon + 360.0
                        } else {
                            lon
                        }
                    })
                    .collect()
            }
            LatLngProjection::LambertConformal(projection) => projection.x.clone().collect(),
            LatLngProjection::Gaussian(projection) => projection.longitudes.clone().collect(),
        }
    }

    pub fn y(&self) -> Vec<f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.latitudes.clone().collect(),
            LatLngProjection::LambertConformal(projection) => projection.y.clone().collect(),
            LatLngProjection::Gaussian(projection) => projection.latitudes.clone().collect(),
        }
    }

    pub fn project_xy(&self, x: f64, y: f64) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(_) => (x, y),
            LatLngProjection::LambertConformal(projection) => {
                let projected = projection.projection.project(x, y).unwrap();
                (projected.1, projected.0)
            }
            LatLngProjection::Gaussian(_) => (x, y),
        }
    }

    pub fn project_latlng(&self, lat: f64, lng: f64) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(_) => (lng, lat),
            LatLngProjection::LambertConformal(projection) => {
                let projected = projection.projection.inverse_project(lng, lat).unwrap();
                (projected.1, projected.0)
            }
            LatLngProjection::Gaussian(_) => (lng, lat),
        }
    }

    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        match self {
            LatLngProjection::PlateCaree(_) | LatLngProjection::LambertConformal(_) => {
                // Use lat_lng() to get normalized coordinates
                let (lat, lng) = self.lat_lng();
                let (min_lat, max_lat) = lat.into_iter().minmax().into_option().unwrap();
                let (min_lng, max_lng) = lng.into_iter().minmax().into_option().unwrap();
                (min_lng, min_lat, max_lng, max_lat)
            }
            LatLngProjection::Gaussian(projection) => {
                let minmax_lat = projection.latitudes.clone().minmax();
                let (min_lat, max_lat) = minmax_lat.into_option().unwrap();
                let minmax_lng = projection.longitudes.clone().minmax();
                let (min_lng, max_lng) = minmax_lng.into_option().unwrap();
                (min_lng, min_lat, max_lng, max_lat)
            }
        }
    }

    pub fn latlng_start(&self) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => (projection.latitudes.start, projection.longitudes.start),
            LatLngProjection::LambertConformal(projection) => {
                self.project_xy(projection.x.start, projection.y.start)
            },
            LatLngProjection::Gaussian(projection) => {
                let first_lat = projection.latitudes.values().first().copied().unwrap_or(0.0);
                (first_lat, projection.longitudes.start)
            },
        }
    }

    pub fn latlng_end(&self) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => (projection.latitudes.end, projection.longitudes.end),
            LatLngProjection::LambertConformal(projection) => {
                self.project_xy(projection.x.end, projection.y.end)
            },
            LatLngProjection::Gaussian(projection) => {
                let last_lat = projection.latitudes.values().last().copied().unwrap_or(0.0);
                (last_lat, projection.longitudes.end)
            },
        }
    }

    pub fn proj_name(&self) -> String {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.projection_name.clone(),
            LatLngProjection::LambertConformal(projection) => projection.projection_name.clone(),
            LatLngProjection::Gaussian(projection) => projection.projection_name.clone(),
        }
    }

    pub fn proj_params(&self) -> HashMap<String, f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.projection_params.clone(),
            LatLngProjection::LambertConformal(projection) => projection.projection_params.clone(),
            LatLngProjection::Gaussian(projection) => projection.projection_params.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegularCoordinateIterator {
    pub start: f64,
    pub step: f64,
    pub end: f64,
    pub current_index: usize,
    pub count: usize,
}

impl RegularCoordinateIterator {
    pub fn new(start: f64, step: f64, count: usize) -> Self {
        Self {
            start,
            step,
            end: start + (step * (count - 1) as f64),
            current_index: 0,
            count,
        }
    }
}

impl Iterator for RegularCoordinateIterator {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            return None;
        } else if self.current_index == self.count {
            return None;
        }

        // Grab the head
        let coordinate = self.start + self.step * self.current_index as f64;

        // Increment the iterator
        self.current_index += 1;

        Some(coordinate)
    }
}

/// Iterator for irregularly-spaced coordinates (e.g., Gaussian latitudes)
#[derive(Clone, Debug)]
pub struct IrregularCoordinateIterator {
    values: Vec<f64>,
    current_index: usize,
}

impl IrregularCoordinateIterator {
    pub fn new(values: Vec<f64>) -> Self {
        Self {
            values,
            current_index: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn values(&self) -> &[f64] {
        &self.values
    }
}

impl Iterator for IrregularCoordinateIterator {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.values.len() {
            return None;
        }

        let value = self.values[self.current_index];
        self.current_index += 1;
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    #[test]
    fn test_regular_grid_iterator() {
        let iter = super::RegularCoordinateIterator::new(0.0, 1.0, 10);
        let coords = iter.collect::<Vec<_>>();
        assert_eq!(coords.len(), 10);
        assert!((coords[0] - 0.0).abs() < f64::EPSILON);
        assert!((coords[9] - 9.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_regular_grid_latlng_projection() {
        let latitudes = super::RegularCoordinateIterator::new(0.0, 1.0, 10);
        let longitudes = super::RegularCoordinateIterator::new(0.0, 2.0, 5);
        let projection = super::LatLngProjection::PlateCaree(crate::utils::iter::projection::PlateCareeProjection { 
            latitudes, 
            longitudes,
            projection_name: "latlon".into(), 
            projection_params: HashMap::from([("a".into(), 6367470.), ("b".into(), 6367470.)]),
        });
        let (lats, lngs) = projection.lat_lng();
        assert_eq!(lats.len(), 10);
        assert_eq!(lngs.len(), 5);
    }

    #[test]
    fn test_start_end_regular_grid() {
        let start = 0.0;
        let step = 0.25;
        let end = 359.75;
        let count = 1440;

        let iter = super::RegularCoordinateIterator::new(start, step, count);
        assert!((iter.end - end).abs() < f64::EPSILON);
    }

    #[test]
    fn test_wrapped_longitude_grid() {
        // Test ECMWF-style grid that starts at 180° and wraps around the globe
        // Grid: 180°, 180.25°, ..., 359.75°, 0°, 0.25°, ..., 179.75°
        let latitudes = super::RegularCoordinateIterator::new(90.0, -0.25, 721);
        let longitudes = super::RegularCoordinateIterator::new(180.0, 0.25, 1440);
        let projection = super::LatLngProjection::PlateCaree(crate::utils::iter::projection::PlateCareeProjection {
            latitudes,
            longitudes,
            projection_name: "latlon".into(),
            projection_params: HashMap::from([("a".into(), 6367470.), ("b".into(), 6367470.)]),
        });

        let (lats, lngs) = projection.lat_lng();

        // Check latitudes are correct
        assert_eq!(lats.len(), 721);
        assert!((lats[0] - 90.0).abs() < f64::EPSILON);
        assert!((lats[720] - (-90.0)).abs() < f64::EPSILON);

        // Check longitudes are normalized
        assert_eq!(lngs.len(), 1440);
        assert!((lngs[0] - 180.0).abs() < f64::EPSILON);  // Starts at 180
        assert!((lngs[720] - 0.0).abs() < f64::EPSILON);   // Wraps to 0 at index 720
        assert!((lngs[1439] - 179.75).abs() < f64::EPSILON); // Ends at 179.75

        // All longitudes should be in [0, 360) range
        for lng in &lngs {
            assert!(*lng >= 0.0 && *lng < 360.0, "Longitude {} out of range", lng);
        }
    }
}
