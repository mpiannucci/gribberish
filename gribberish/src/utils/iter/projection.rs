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
            LatLngProjection::PlateCaree(projection) => {
                let lats: Vec<f64> = projection.latitudes.clone().collect();
                let lon_start = projection.longitudes.start;
                let lngs: Vec<f64> = projection
                    .longitudes
                    .clone()
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
        }
    }

    /// Columns to roll this projection's longitude axis (and matching data) left
    /// so longitudes run monotonically over `[-180, 180)`, or `None` for
    /// projected/ineligible grids (callers no-op). See [`wrap_roll`] for the
    /// eligibility rules and how the roll is chosen.
    fn longitude_wrap_roll(&self) -> Option<usize> {
        match self {
            // For a regular grid `lat_lng().1` is the 1-D longitude axis; for a
            // projected grid it is the full flattened field, which is never
            // eligible, so short-circuit it.
            LatLngProjection::PlateCaree(_) => wrap_roll(&self.lat_lng().1),
            LatLngProjection::LambertConformal(_) => None,
        }
    }

    /// Whether the decoded rows (and the row-defining coordinate) need to be
    /// reversed to put the northern-most row first. The decision is read off the
    /// sign of the row-defining axis step: a positive step means the axis
    /// ascends, so row 0 is the southern-most and a flip is needed; a
    /// non-positive step means the grid is already north-first and this is a
    /// no-op. For a regular grid the row axis is the latitude axis; for a Lambert
    /// Conformal grid it is the projected `y` (northing) axis, since latitude is
    /// a 2-D field there.
    fn needs_north_up_flip(&self) -> bool {
        match self {
            LatLngProjection::PlateCaree(p) => p.latitudes.step > 0.0,
            LatLngProjection::LambertConformal(p) => p.y.step > 0.0,
        }
    }

    /// Grid dimensions as `(ny, nx)` — rows (the row-defining / latitude / `y`
    /// axis) by columns (the longitude / `x` axis) — for the row-major `ny × nx`
    /// layout shared by the decoded data and the flattened coordinate fields.
    fn dims(&self) -> (usize, usize) {
        match self {
            LatLngProjection::PlateCaree(p) => (p.latitudes.count, p.longitudes.count),
            LatLngProjection::LambertConformal(p) => (p.y.count, p.x.count),
        }
    }

    /// Like [`lat_lng`](Self::lat_lng), but applies the two opt-in on-the-fly
    /// coordinate adjustments.
    ///
    /// When `adjust_longitude_range` is set the longitudes are wrapped to
    /// `[-180, 180)` and rotated so they are monotonically increasing (see
    /// [`adjust_longitude_values`]); this permutes columns within each row.
    ///
    /// When `north_up` is set the rows are reordered so the 0th row is the
    /// northern-most (see [`needs_north_up_flip`](Self::needs_north_up_flip));
    /// this permutes whole rows. For a regular grid only the 1-D latitude axis is
    /// reversed (the longitude axis is unaffected by a row flip); for a Lambert
    /// Conformal grid both the flattened latitude and longitude fields are
    /// row-reversed.
    ///
    /// The two adjustments are orthogonal and may be combined. Each is a no-op
    /// (identical to `lat_lng`) when its flag is false or the grid is not
    /// eligible.
    pub fn lat_lng_adjusted(
        &self,
        adjust_longitude_range: bool,
        north_up: bool,
    ) -> (Vec<f64>, Vec<f64>) {
        let (mut lats, mut lngs) = self.lat_lng();
        if adjust_longitude_range {
            lngs = adjust_longitude_values(lngs);
        }
        if north_up && self.needs_north_up_flip() {
            match self {
                // 1-D lat axis; lng axis unaffected by a row flip.
                LatLngProjection::PlateCaree(_) => {
                    lats.reverse();
                }
                LatLngProjection::LambertConformal(_) => {
                    let (ny, nx) = self.dims();
                    lats = reverse_rows(&lats, ny, nx);
                    lngs = reverse_rows(&lngs, ny, nx);
                }
            }
        }
        (lats, lngs)
    }

    /// Apply the same longitude wrap as [`lat_lng_adjusted`](Self::lat_lng_adjusted)
    /// to a decoded data buffer laid out row-major as `ny × nx` (latitude-major,
    /// longitude fastest), rolling its columns so it stays aligned with the
    /// wrapped longitude coordinates. A no-op when `adjust` is false or the grid
    /// is not eligible.
    pub fn adjust_data_longitude(&self, data: Vec<f64>, adjust: bool) -> Vec<f64> {
        if !adjust {
            return data;
        }
        match (self, self.longitude_wrap_roll()) {
            (LatLngProjection::PlateCaree(_), Some(roll)) => {
                let (ny, nx) = self.dims();
                rotate_rows_left(&data, ny, nx, roll)
            }
            _ => data,
        }
    }

    /// Reorder a decoded data buffer laid out row-major as `ny × nx`
    /// (latitude-major, longitude fastest) so the northern-most row comes first,
    /// keeping it aligned with the latitudes produced by
    /// [`lat_lng_adjusted`](Self::lat_lng_adjusted) when `north_up` is set. A
    /// no-op when `north_up` is false or the grid is already north-first (see
    /// [`needs_north_up_flip`](Self::needs_north_up_flip)).
    pub fn adjust_data_north_up(&self, data: Vec<f64>, north_up: bool) -> Vec<f64> {
        if !north_up || !self.needs_north_up_flip() {
            return data;
        }
        let (ny, nx) = self.dims();
        reverse_rows(&data, ny, nx)
    }

    pub fn x(&self) -> Vec<f64> {
        match self {
            LatLngProjection::PlateCaree(projection) => {
                let lon_start = projection.longitudes.start;
                projection
                    .longitudes
                    .clone()
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
            LatLngProjection::PlateCaree(_) | LatLngProjection::LambertConformal(_) => {
                // Use lat_lng() to get normalized coordinates
                let (lat, lng) = self.lat_lng();
                let (min_lat, max_lat) = lat.into_iter().minmax().into_option().unwrap();
                let (min_lng, max_lng) = lng.into_iter().minmax().into_option().unwrap();
                (min_lng, min_lat, max_lng, max_lat)
            }
        }
    }

    pub fn latlng_start(&self) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => {
                (projection.latitudes.start, projection.longitudes.start)
            }
            LatLngProjection::LambertConformal(projection) => {
                self.project_xy(projection.x.start, projection.y.start)
            }
        }
    }

    pub fn latlng_end(&self) -> (f64, f64) {
        match self {
            LatLngProjection::PlateCaree(projection) => {
                (projection.latitudes.end, projection.longitudes.end)
            }
            LatLngProjection::LambertConformal(projection) => {
                self.project_xy(projection.x.end, projection.y.end)
            }
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
            end: start + (step * (count - 1) as f64),
            current_index: 0,
            count,
        }
    }
}

impl Iterator for RegularCoordinateIterator {
    type Item = f64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 || self.current_index == self.count {
            return None;
        }

        // Grab the head
        let coordinate = self.start + self.step * self.current_index as f64;

        // Increment the iterator
        self.current_index += 1;

        Some(coordinate)
    }
}

/// Wrap a longitude given in `[0, 360)` into `[-180, 180)`. The antimeridian
/// (exactly 180°) maps to -180.
fn wrap_longitude(lon: f64) -> f64 {
    if lon >= 180.0 {
        lon - 360.0
    } else {
        lon
    }
}

/// For an ascending, near-global `[0, 360)` longitude axis, the number of
/// columns to rotate left so the wrapped `[-180, 180)` axis is monotonic.
///
/// Returns `None` (callers should no-op) unless the axis spans ~360° with an
/// ascending step. The roll is the index of the column nearest the antimeridian
/// from the east (the most-negative wrapped longitude); a grid that already
/// starts at 180° rolls by `0` (relabel only, no data move). The step is read
/// off the first two samples, so this works on either a projector's longitude
/// axis or an already-materialized longitude coordinate.
///
/// The logic to determine whether this should be applied mirrors GDAL's
/// `GRIB_ADJUST_LONGITUDE_RANGE` "split & swap" option.
fn wrap_roll(lons: &[f64]) -> Option<usize> {
    let nx = lons.len();
    if nx < 2 {
        return None;
    }
    // Regular ascending grid only; descending-longitude grids are out of scope.
    let dx = lons[1] - lons[0];
    if dx <= 0.0 {
        return None;
    }
    // Near-global: the columns must densely cover ~360°, within a quarter cell
    // (GDAL's tolerance). Excludes regional subsets and overlapping/duplicated
    // wrap-around columns (e.g. a grid carrying both 0° and 360°).
    if (dx * nx as f64 - 360.0).abs() >= dx / 4.0 {
        return None;
    }

    // Rotate so the column with the most-negative wrapped longitude (closest to
    // -180 from above) leads, which makes the wrapped axis monotonic.
    let (roll, _) =
        lons.iter()
            .enumerate()
            .fold((0usize, f64::INFINITY), |(roll, min), (i, &lon)| {
                let wrapped = wrap_longitude(lon);
                if wrapped < min {
                    (i, wrapped)
                } else {
                    (roll, min)
                }
            });
    Some(roll)
}

/// Wrap and rotate a `[0, 360)` longitude coordinate axis to a monotonic
/// `[-180, 180)` range, returning it unchanged for axes that aren't eligible
/// near-global ascending grids (see [`wrap_roll`]). The matching data buffer
/// must be rolled with [`LatLngProjection::adjust_data_longitude`] so the
/// coordinate and the data stay aligned.
pub fn adjust_longitude_values(longitudes: Vec<f64>) -> Vec<f64> {
    match wrap_roll(&longitudes) {
        Some(roll) => {
            let wrapped: Vec<f64> = longitudes.iter().map(|&lon| wrap_longitude(lon)).collect();
            rotate_left(&wrapped, roll)
        }
        None => longitudes,
    }
}

/// Reverse a regular 1-D latitude axis so it runs north-to-south (descending),
/// returning it unchanged if it is already descending or too short to tell. The
/// matching data buffer must be row-reversed with
/// [`LatLngProjection::adjust_data_north_up`] so the coordinate and the data
/// stay aligned. This is the latitude sibling of [`adjust_longitude_values`],
/// used by the parser to flip an inlined 1-D latitude coordinate.
pub fn adjust_latitude_values(latitudes: Vec<f64>) -> Vec<f64> {
    if latitudes.len() >= 2 && latitudes[1] > latitudes[0] {
        let mut v = latitudes;
        v.reverse();
        v
    } else {
        latitudes
    }
}

/// Rotate a slice left by `roll` positions: `out[i] = values[(i + roll) % n]`.
fn rotate_left(values: &[f64], roll: usize) -> Vec<f64> {
    let n = values.len();
    if n == 0 || roll.is_multiple_of(n) {
        return values.to_vec();
    }
    let roll = roll % n;
    (0..n).map(|i| values[(i + roll) % n]).collect()
}

/// Rotate each row of a row-major `ny × nx` buffer left by `roll` columns. Used
/// to apply a longitude wrap to decoded data so it stays aligned with the
/// wrapped longitude coordinates. Returns the input unchanged if `roll` is a
/// no-op or the dimensions don't match the buffer length.
fn rotate_rows_left(data: &[f64], ny: usize, nx: usize, roll: usize) -> Vec<f64> {
    if nx == 0 || roll.is_multiple_of(nx) || ny * nx != data.len() {
        return data.to_vec();
    }
    let roll = roll % nx;
    let mut out = vec![0.0; data.len()];
    for r in 0..ny {
        let row = &data[r * nx..(r + 1) * nx];
        for c in 0..nx {
            out[r * nx + c] = row[(c + roll) % nx];
        }
    }
    out
}

/// Reverse the row order of a row-major `ny × nx` buffer, leaving the columns
/// within each row untouched. Used to apply a north-up flip to decoded data so
/// it stays aligned with the row-reversed coordinates. Returns the input
/// unchanged if the dimensions don't match the buffer length.
fn reverse_rows(data: &[f64], ny: usize, nx: usize) -> Vec<f64> {
    if nx == 0 || ny * nx != data.len() {
        return data.to_vec();
    }
    // `rchunks(nx)` yields the rows back-to-front; flatten them into the output.
    data.rchunks(nx).flatten().copied().collect()
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
        let projection = super::LatLngProjection::PlateCaree(
            crate::utils::iter::projection::PlateCareeProjection {
                latitudes,
                longitudes,
                projection_name: "latlon".into(),
                projection_params: HashMap::from([("a".into(), 6367470.), ("b".into(), 6367470.)]),
            },
        );
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
        let projection = super::LatLngProjection::PlateCaree(
            crate::utils::iter::projection::PlateCareeProjection {
                latitudes,
                longitudes,
                projection_name: "latlon".into(),
                projection_params: HashMap::from([("a".into(), 6367470.), ("b".into(), 6367470.)]),
            },
        );

        let (lats, lngs) = projection.lat_lng();

        // Check latitudes are correct
        assert_eq!(lats.len(), 721);
        assert!((lats[0] - 90.0).abs() < f64::EPSILON);
        assert!((lats[720] - (-90.0)).abs() < f64::EPSILON);

        // Check longitudes are normalized
        assert_eq!(lngs.len(), 1440);
        assert!((lngs[0] - 180.0).abs() < f64::EPSILON); // Starts at 180
        assert!((lngs[720] - 0.0).abs() < f64::EPSILON); // Wraps to 0 at index 720
        assert!((lngs[1439] - 179.75).abs() < f64::EPSILON); // Ends at 179.75

        // All longitudes should be in [0, 360) range
        for lng in &lngs {
            assert!(
                *lng >= 0.0 && *lng < 360.0,
                "Longitude {} out of range",
                lng
            );
        }
    }

    fn platecaree(lon_start: f64, lon_step: f64, lon_count: usize) -> super::LatLngProjection {
        platecaree_grid(90.0, -1.0, 5, lon_start, lon_step, lon_count)
    }

    /// Like [`platecaree`], but with a fully configurable latitude axis so tests
    /// can build south-first (ascending) grids to exercise the north-up flip.
    fn platecaree_grid(
        lat_start: f64,
        lat_step: f64,
        lat_count: usize,
        lon_start: f64,
        lon_step: f64,
        lon_count: usize,
    ) -> super::LatLngProjection {
        let latitudes = super::RegularCoordinateIterator::new(lat_start, lat_step, lat_count);
        let longitudes = super::RegularCoordinateIterator::new(lon_start, lon_step, lon_count);
        super::LatLngProjection::PlateCaree(crate::utils::iter::projection::PlateCareeProjection {
            latitudes,
            longitudes,
            projection_name: "latlon".into(),
            projection_params: HashMap::new(),
        })
    }

    #[test]
    fn test_longitude_wrap_roll() {
        // GFS 0.25°: 1440 columns, split at 180° -> column 720.
        assert_eq!(platecaree(0.0, 0.25, 1440).longitude_wrap_roll(), Some(720));
        // GEFS 0.5°: 720 columns -> column 360.
        assert_eq!(platecaree(0.0, 0.5, 720).longitude_wrap_roll(), Some(360));
        // ERA5-style 3° GRIB: 120 columns, 180° at column 60.
        assert_eq!(platecaree(0.0, 3.0, 120).longitude_wrap_roll(), Some(60));
        // ECMWF grid that already starts at 180°: relabel only, no data move.
        assert_eq!(platecaree(180.0, 0.25, 1440).longitude_wrap_roll(), Some(0));
        // Grid already in [-180, 180): nothing to roll.
        assert_eq!(
            platecaree(-180.0, 0.25, 1440).longitude_wrap_roll(),
            Some(0)
        );
    }

    #[test]
    fn test_longitude_wrap_roll_ineligible() {
        // Regional subset (90° wide) is not global -> None.
        assert_eq!(platecaree(0.0, 0.25, 360).longitude_wrap_roll(), None);
        // Descending longitude axis is out of scope -> None.
        assert_eq!(platecaree(359.75, -0.25, 1440).longitude_wrap_roll(), None);
        // Overlapping wrap-around (carries both 0° and 360°) -> None.
        assert_eq!(platecaree(0.0, 0.25, 1441).longitude_wrap_roll(), None);
    }

    #[test]
    fn test_lat_lng_adjusted_monotonic() {
        let projection = platecaree(0.0, 0.5, 720);
        let (_, native) = projection.lat_lng();
        let (lats, lngs) = projection.lat_lng_adjusted(true, false);

        // Latitudes are untouched.
        assert_eq!(lats, projection.lat_lng().0);

        // Strictly monotonic increasing over [-180, 180).
        assert_eq!(lngs.len(), 720);
        assert!((lngs[0] - (-180.0)).abs() < f64::EPSILON);
        assert!((lngs[719] - 179.5).abs() < f64::EPSILON);
        for w in lngs.windows(2) {
            assert!(w[1] > w[0], "not monotonic at {} -> {}", w[0], w[1]);
        }
        for &lng in &lngs {
            assert!((-180.0..180.0).contains(&lng), "out of range: {lng}");
        }

        // No data lost: the wrapped set matches wrapping each native value.
        let mut from_native: Vec<f64> = native.iter().map(|&l| super::wrap_longitude(l)).collect();
        let mut adjusted = lngs.clone();
        from_native.sort_by(|a, b| a.partial_cmp(b).unwrap());
        adjusted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(from_native, adjusted);
    }

    #[test]
    fn test_longitude_wrap_off_column_split() {
        // Cell-centered 1° grid (0.5, 1.5, ... 359.5): 180° falls *between*
        // columns, so the split must land on the most-negative wrapped column
        // (180.5 -> -179.5 at index 180), not on an exact 180° column.
        let projection = platecaree(0.5, 1.0, 360);
        assert_eq!(projection.longitude_wrap_roll(), Some(180));

        let (_, lngs) = projection.lat_lng_adjusted(true, false);
        assert_eq!(lngs.len(), 360);
        assert!((lngs[0] - (-179.5)).abs() < f64::EPSILON);
        assert!((lngs[359] - 179.5).abs() < f64::EPSILON);
        for w in lngs.windows(2) {
            assert!(w[1] > w[0], "not monotonic at {} -> {}", w[0], w[1]);
        }
        for &lng in &lngs {
            assert!((-180.0..180.0).contains(&lng), "out of range: {lng}");
        }

        // Idempotent: wrapping an already-monotonic [-180, 180) axis is a no-op
        // (its roll is 0).
        assert_eq!(super::adjust_longitude_values(lngs.clone()), lngs);
    }

    #[test]
    fn test_lat_lng_adjusted_noop_when_disabled_or_ineligible() {
        let global = platecaree(0.0, 0.5, 720);
        assert_eq!(global.lat_lng_adjusted(false, false), global.lat_lng());

        let regional = platecaree(0.0, 0.25, 360);
        assert_eq!(regional.lat_lng_adjusted(true, false), regional.lat_lng());
    }

    #[test]
    fn test_rotate_rows_left() {
        // 2 rows x 4 cols, roll left by 1.
        let data = vec![0.0, 1.0, 2.0, 3.0, 10.0, 11.0, 12.0, 13.0];
        let rolled = super::rotate_rows_left(&data, 2, 4, 1);
        assert_eq!(rolled, vec![1.0, 2.0, 3.0, 0.0, 11.0, 12.0, 13.0, 10.0]);

        // roll == 0 and roll == nx are no-ops.
        assert_eq!(super::rotate_rows_left(&data, 2, 4, 0), data);
        assert_eq!(super::rotate_rows_left(&data, 2, 4, 4), data);

        // Mismatched dimensions return the input unchanged.
        assert_eq!(super::rotate_rows_left(&data, 3, 4, 1), data);
    }

    #[test]
    fn test_needs_north_up_flip() {
        // South-first (ascending) latitude axis: row 0 is the southern-most, so a
        // flip is needed to put north first.
        assert!(platecaree_grid(-90.0, 1.0, 5, 0.0, 1.0, 4).needs_north_up_flip());
        // North-first (descending) latitude axis: already north-first, no flip.
        assert!(!platecaree_grid(90.0, -1.0, 5, 0.0, 1.0, 4).needs_north_up_flip());
    }

    #[test]
    fn test_reverse_rows() {
        // 3 rows x 2 cols: reverse the row order, columns within each row intact.
        let data = vec![0.0, 1.0, 10.0, 11.0, 20.0, 21.0];
        let reversed = super::reverse_rows(&data, 3, 2);
        assert_eq!(reversed, vec![20.0, 21.0, 10.0, 11.0, 0.0, 1.0]);

        // Mismatched dimensions return the input unchanged.
        assert_eq!(super::reverse_rows(&data, 4, 2), data);
        // Zero-width buffer returns the input unchanged.
        assert_eq!(super::reverse_rows(&data, 0, 0), data);
    }

    #[test]
    fn test_adjust_data_north_up() {
        // 3 rows x 2 cols of data aligned with the latitude axis row-for-row.
        let data = vec![0.0, 1.0, 10.0, 11.0, 20.0, 21.0];

        // South-first grid: rows are reversed so the northern-most row leads.
        let south_first = platecaree_grid(-90.0, 1.0, 3, 0.0, 1.0, 2);
        assert_eq!(
            south_first.adjust_data_north_up(data.clone(), true),
            vec![20.0, 21.0, 10.0, 11.0, 0.0, 1.0]
        );
        // ...but only when north_up is requested.
        assert_eq!(south_first.adjust_data_north_up(data.clone(), false), data);

        // North-first grid: already north-first, so it is a no-op even when set.
        let north_first = platecaree_grid(90.0, -1.0, 3, 0.0, 1.0, 2);
        assert_eq!(north_first.adjust_data_north_up(data.clone(), true), data);
    }

    #[test]
    fn test_lat_lng_adjusted_north_up() {
        // South-first axis: lat_lng_adjusted(_, true) reverses the latitude axis
        // so it runs north-to-south (descending); longitudes are untouched.
        let south_first = platecaree_grid(-90.0, 1.0, 5, 0.0, 1.0, 4);
        let (lats, lngs) = south_first.lat_lng_adjusted(false, true);
        for w in lats.windows(2) {
            assert!(
                w[1] < w[0],
                "latitudes should descend: {} -> {}",
                w[0],
                w[1]
            );
        }
        assert!((lats[0] - (-86.0)).abs() < f64::EPSILON);
        assert!((lats[4] - (-90.0)).abs() < f64::EPSILON);
        assert_eq!(lngs, south_first.lat_lng().1);

        // North-first axis: already north-first, so north_up leaves it unchanged.
        let north_first = platecaree_grid(90.0, -1.0, 5, 0.0, 1.0, 4);
        assert_eq!(
            north_first.lat_lng_adjusted(false, true),
            north_first.lat_lng()
        );
    }

    #[test]
    fn test_adjust_latitude_values() {
        // Ascending (south-first) axis is reversed to descending (north-first).
        assert_eq!(
            super::adjust_latitude_values(vec![-90.0, -45.0, 0.0, 45.0, 90.0]),
            vec![90.0, 45.0, 0.0, -45.0, -90.0]
        );
        // Already-descending axis is left unchanged.
        let descending = vec![90.0, 45.0, 0.0, -45.0, -90.0];
        assert_eq!(
            super::adjust_latitude_values(descending.clone()),
            descending
        );
        // Idempotent: reversing an already-descending axis is a no-op.
        let once = super::adjust_latitude_values(vec![-90.0, 0.0, 90.0]);
        assert_eq!(super::adjust_latitude_values(once.clone()), once);
        // Too short to tell: returned unchanged.
        assert_eq!(super::adjust_latitude_values(vec![5.0]), vec![5.0]);
    }

    #[test]
    fn test_north_up_and_longitude_wrap_composable() {
        // Synthetic south-first global grid: ascending latitudes (flip needed) and
        // an ascending [0, 360) longitude axis (wrap/roll needed). The two
        // adjustments are orthogonal: north_up permutes rows, the wrap rolls
        // columns, and both can be applied together.
        let ny = 4usize;
        let nx = 4usize; // 0, 90, 180, 270 -> rolls by 2 to lead with -180.
        let projection = platecaree_grid(-90.0, 60.0, ny, 0.0, 90.0, nx);

        // Coordinates: latitudes reversed (descending) and longitudes wrapped.
        let (lats, lngs) = projection.lat_lng_adjusted(true, true);
        for w in lats.windows(2) {
            assert!(w[1] < w[0], "latitudes should descend");
        }
        assert!((lngs[0] - (-180.0)).abs() < f64::EPSILON);
        for w in lngs.windows(2) {
            assert!(w[1] > w[0], "longitudes should ascend monotonically");
        }

        // Data: build a buffer whose value encodes (row, col) so we can verify
        // both the row reversal and the column roll happened together.
        let data: Vec<f64> = (0..ny)
            .flat_map(|r| (0..nx).map(move |c| (r * 10 + c) as f64))
            .collect();
        let rolled = projection.adjust_data_longitude(data.clone(), true);
        let both = projection.adjust_data_north_up(rolled, true);
        // Expected: rows reversed, then within each row columns rolled left by 2.
        let mut expected = Vec::with_capacity(data.len());
        for r in (0..ny).rev() {
            for c in 0..nx {
                expected.push((r * 10 + (c + 2) % nx) as f64);
            }
        }
        assert_eq!(both, expected);
    }
}
