pub trait GridDefinitionTemplate {
    fn proj_string(&self) -> String;
    fn crs(&self) -> String;
    fn grid_point_count(&self) -> usize;
    fn is_regular_grid(&self) -> bool;
    fn latitude_count(&self) -> usize;
    fn longitude_count(&self) -> usize;
    fn latlng(&self) -> Vec<(f64, f64)>;

    fn bbox(&self) -> (f64, f64, f64, f64) {
        let mut min_lat = 180.0;
        let mut max_lat = -180.0;
        let mut min_lng = 361.0;
        let mut max_lng = -360.0;
        self.latlng().iter().for_each(|(lat, lng)| {
            if *lat < min_lat {
                min_lat = *lat;
            }
            if *lat > max_lat {
                max_lat = *lat;
            }

            if *lng < min_lng {
                min_lng = *lng;
            }
            if *lng > max_lng {
                max_lng = *lng;
            }
        });

        (min_lng, min_lat, max_lng, max_lat)
    }

    fn grid_bounds(&self) -> ((f64, f64), (f64, f64)) {
        let bbox = self.bbox();
        ((bbox.1, bbox.0), (bbox.3, bbox.2))
    }

    fn index_for_indices(&self, indices: (usize, usize)) -> usize {
        (indices.0 * self.longitude_count()) + indices.1
    }

    fn latlng_values(&self) -> (Vec<f64>, Vec<f64>) {
        self.latlng().into_iter().unzip()
    }

    fn latlng_in_range(
        &self,
        latitude_range: (f64, f64),
        longitude_range: (f64, f64),
    ) -> Vec<(usize, (f64, f64))> {
        self.latlng()
            .into_iter()
            .enumerate()
            .filter(|(_, l)| {
                l.0 > latitude_range.0
                    && l.0 < latitude_range.1
                    && l.1 > longitude_range.0
                    && l.1 < longitude_range.1
            })
            .collect()
    }

    fn latlng_grid(&self) -> Vec<Vec<Vec<f64>>> {
        let latlng = self.latlng();
        (0..self.latitude_count())
            .map(|lat_i| {
                (0..self.longitude_count())
                    .map(|lon_i| {
                        let idx = lat_i * self.longitude_count() + lon_i;
                        let (lat, lng) = latlng[idx];
                        vec![lat, lng]
                    })
                    .collect()
            })
            .collect()
    }

    fn zerod_location_grid(&self) -> Vec<Vec<f64>> {
        let longitude_count = self.longitude_count();
        (0..self.latitude_count())
            .map(|_| (0..longitude_count).map(|_| 0.0).collect())
            .collect()
    }
}
