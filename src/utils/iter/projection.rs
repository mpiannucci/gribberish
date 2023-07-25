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

#[derive(Clone, Debug)]
pub enum LatLngProjection {
    PlateCaree(PlateCareeProjection),
    LambertConformal(LambertConformalConicProjection),
}

impl LatLngProjection {
    pub fn is_regular_latlng_grid(&self) -> bool {
        match self {
            LatLngProjection::PlateCaree(_) => true,
            LatLngProjection::LambertConformal(_) => false,
        }
    }

    pub fn lat_lng(&self) -> (Vec<f64>, Vec<f64>) {
        match self {
            LatLngProjection::PlateCaree(projection) => (
                projection.latitudes.clone().collect(),
                projection.longitudes.clone().collect(),
            ),
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
        }
    }

    pub fn x(&self) -> Vec<f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.longitudes.clone().collect(),
            LatLngProjection::LambertConformal(projection) => projection.x.clone().collect(),
        }
    }

    pub fn y(&self) -> Vec<f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.latitudes.clone().collect(),
            LatLngProjection::LambertConformal(projection) => projection.y.clone().collect(),
        }
    }

    pub fn project_xy(&self, x: f64, y: f64) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(_) => (x, y),
            LatLngProjection::LambertConformal(projection) => {
                let projected = projection.projection.project(x, y).unwrap();
                (projected.1, projected.0)
            }
        }
    }

    pub fn project_latlng(&self, lat: f64, lng: f64) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(_) => (lng, lat),
            LatLngProjection::LambertConformal(projection) => {
                let projected = projection.projection.inverse_project(lng, lat).unwrap();
                (projected.1, projected.0)
            }
        }
    }

    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => {
                let minmax_lat = projection.latitudes.clone().minmax();
                let (min_lat, max_lat) = minmax_lat.into_option().unwrap();
                let minmax_lng = projection.longitudes.clone().minmax();
                let (min_lng, max_lng) = minmax_lng.into_option().unwrap();
                (min_lat, min_lng, max_lat, max_lng)
            }
            LatLngProjection::LambertConformal(_) => {
                let (lat, lng) = self.lat_lng();
                let (min_lat, max_lat) = lat.into_iter().minmax().into_option().unwrap();
                let (min_lng, max_lng) = lng.into_iter().minmax().into_option().unwrap();
                (min_lat, min_lng, max_lat, max_lng)
            }
        }
    }

    pub fn latlng_start(&self) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => (projection.latitudes.start, projection.longitudes.start),
            LatLngProjection::LambertConformal(projection) => {
                self.project_xy(projection.x.start, projection.y.start)
            },
        }
    }

    pub fn latlng_end(&self) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => (projection.latitudes.end, projection.longitudes.end),
            LatLngProjection::LambertConformal(projection) => {
                self.project_xy(projection.x.end, projection.y.end)
            },
        }
    }

    pub fn proj_name(&self) -> String {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.projection_name.clone(),
            LatLngProjection::LambertConformal(projection) => projection.projection_name.clone(),
        }
    }

    pub fn proj_params(&self) -> HashMap<String, f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => projection.projection_params.clone(),
            LatLngProjection::LambertConformal(projection) => projection.projection_params.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegularCoordinateIterator {
    start: f64,
    step: f64,
    end: f64,
    current_index: usize,
    count: usize,
}

impl RegularCoordinateIterator {
    pub fn new(start: f64, step: f64, count: usize) -> Self {
        Self {
            start,
            step,
            end: start + step * count as f64,
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
}
