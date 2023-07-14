use itertools::Itertools;
use mappers::{projections::LambertConformalConic, Projection};

#[derive(Clone, Debug)]
pub enum LatLngProjection {
    PlateCaree(RegularCoordinateIterator, RegularCoordinateIterator),
    LambertConformal(
        RegularCoordinateIterator,
        RegularCoordinateIterator,
        LambertConformalConic,
    ),
}

impl LatLngProjection {
    pub fn is_regular_latlng_grid(&self) -> bool {
        match self {
            LatLngProjection::PlateCaree(_, _) => true,
            LatLngProjection::LambertConformal(_, _, _) => false,
        }
    }

    pub fn lat_lng(&self) -> (Vec<f64>, Vec<f64>) {
        match self {
            LatLngProjection::PlateCaree(latitudes, longitudes) => {
                (latitudes.clone().collect(), longitudes.clone().collect())
            }
            LatLngProjection::LambertConformal(y, x, projection) => y
                .clone()
                .flat_map(|y_coord| {
                    let mut x = x.clone();
                    x.current_index = 0;
                    x.clone()
                        .map(|x_coord| {
                            let projected = projection
                                .inverse_project(x_coord, y_coord)
                                .expect("Failed to inverse project from xy to lnglat");
                            (projected.1, projected.0)
                        })
                        .collect::<Vec<(f64, f64)>>()
                })
                .unzip(),
        }
    }

    pub fn bbox(&self) -> (f64, f64, f64, f64) {
        match self {
            LatLngProjection::PlateCaree(latitudes, longitudes) => {
                let minmax_lat = latitudes.clone().minmax();
                let (min_lat, max_lat) = minmax_lat.into_option().unwrap();
                let minmax_lng = longitudes.clone().minmax();
                let (min_lng, max_lng) = minmax_lng.into_option().unwrap();
                (min_lat, min_lng, max_lat, max_lng)
            }
            LatLngProjection::LambertConformal(_, _, _) => {
                let (lat, lng) = self.lat_lng();
                let (min_lat, max_lat) = lat.into_iter().minmax().into_option().unwrap();
                let (min_lng, max_lng) = lng.into_iter().minmax().into_option().unwrap();
                (min_lat, min_lng, max_lat, max_lng)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct RegularCoordinateIterator {
    start: f64,
    step: f64,
    current_index: usize,
    count: usize,
}

impl RegularCoordinateIterator {
    pub fn new(start: f64, step: f64, count: usize) -> Self {
        Self {
            start,
            step,
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
        let lat = super::RegularCoordinateIterator::new(0.0, 1.0, 10);
        let lng = super::RegularCoordinateIterator::new(0.0, 2.0, 5);
        let projection = super::LatLngProjection::PlateCaree(lat, lng);
        let (lats, lngs) = projection.lat_lng();
        assert_eq!(lats.len(), 10);
        assert_eq!(lngs.len(), 5);
    }
}
