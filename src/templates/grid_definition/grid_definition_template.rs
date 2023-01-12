pub trait GridDefinitionTemplate<'a> {
    fn proj_string(&self) -> String;
    fn crs(&self) -> String;
    fn grid_point_count(&self) -> usize;
    fn start(&self) -> (f64, f64);
    fn origin(&self) -> (f64, f64);
    fn end(&self) -> (f64, f64);
    fn latitude_count(&self) -> usize;
    fn longitude_count(&self) -> usize;
    fn latitudes(&self) -> Vec<f64>;
    fn longitudes(&self) -> Vec<f64>;
    fn locations(&self) -> Vec<(f64, f64)>;
    fn location_for_index(&self, index: usize) -> Result<(f64, f64), &'static str>;

    fn index_for_indices(&self, indices: (usize, usize)) -> usize {
        (indices.0 * self.longitude_count()) + indices.1
    }

    fn latitudes_in_range(&self, range: (f64, f64)) -> Vec<(usize, f64)> {
        self.latitudes()
            .into_iter()
            .enumerate()
            .filter(|(_, l)| *l > range.0 && *l < range.1)
            .collect()
    }

    fn longitudes_in_range(&self, range: (f64, f64)) -> Vec<(usize, f64)> {
        self.longitudes()
            .into_iter()
            .enumerate()
            .filter(|(_, l)| *l > range.0 && *l < range.1)
            .collect()
    }

    fn locations_in_range(
        &self,
        latitude_range: (f64, f64),
        longitude_range: (f64, f64),
    ) -> Vec<(usize, (f64, f64))> {
        self.locations()
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

    fn location_grid(&self) -> Vec<Vec<Vec<f64>>> {
        let longitudes = self.longitudes();
        let latitudes = self.latitudes();
        latitudes
            .iter()
            .map(|lat| longitudes
                .iter()
                .map(|lon| vec![*lat, *lon])
                .collect()
            )
            .collect()
    }

    fn zerod_location_grid(&self) -> Vec<Vec<f64>> {
        let longitudes = self.longitudes();
        self.latitudes()
            .into_iter()
            .map(|_| longitudes.iter().map(|_| 0.).collect())
            .collect()
    }
}