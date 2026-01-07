use bitvec::prelude::*;

use super::grid_definition_template::GridDefinitionTemplate;
use super::tables::{EarthShape, ScanningMode, ScanningModeFlags};
use crate::templates::template::{Template, TemplateType};
use crate::utils::iter::projection::{
    GaussianProjection, IrregularCoordinateIterator, LatLngProjection, RegularCoordinateIterator,
};
use crate::utils::read_u32_from_bytes;

use std::collections::HashMap;
use std::f64::consts::PI;
use std::vec::Vec;

/// Grid Definition Template 40 - Gaussian Latitude/Longitude Grid
///
/// A Gaussian grid is similar to a regular lat/lon grid but the latitudes
/// are positioned at Gaussian quadrature points (roots of Legendre polynomials)
/// rather than being evenly spaced.
pub struct GaussianTemplate {
    data: Vec<u8>,
}

impl Template for GaussianTemplate {
    fn template_type(&self) -> TemplateType {
        TemplateType::Grid
    }

    fn template_number(&self) -> u16 {
        40
    }

    fn data(&self) -> &[u8] {
        &self.data
    }

    fn template_name(&self) -> &str {
        "Gaussian Latitude/Longitude"
    }
}

impl GaussianTemplate {
    pub fn new(data: Vec<u8>) -> Self {
        GaussianTemplate { data }
    }

    pub fn earth_shape(&self) -> EarthShape {
        self.data[14].into()
    }

    pub fn earth_radius_scale_factor(&self) -> u8 {
        self.data[15]
    }

    pub fn earth_radius_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 16).unwrap_or(0)
    }

    pub fn earth_major_axis_scale_factor(&self) -> u8 {
        self.data[20]
    }

    pub fn earth_major_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 21).unwrap_or(0)
    }

    pub fn earth_minor_axis_scale_factor(&self) -> u8 {
        self.data[25]
    }

    pub fn earth_minor_axis_scaled_value(&self) -> u32 {
        read_u32_from_bytes(&self.data, 26).unwrap_or(0)
    }

    /// Number of points along a parallel (Ni)
    pub fn parallel_point_count(&self) -> u32 {
        read_u32_from_bytes(&self.data, 30).unwrap_or(0)
    }

    /// Number of points along a meridian (Nj)
    pub fn meridian_point_count(&self) -> u32 {
        read_u32_from_bytes(&self.data, 34).unwrap_or(0)
    }

    pub fn basic_angle(&self) -> u32 {
        read_u32_from_bytes(&self.data, 38).unwrap_or(0)
    }

    pub fn subdivision(&self) -> u32 {
        read_u32_from_bytes(&self.data, 42).unwrap_or(0)
    }

    pub fn start_latitude(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 46).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn start_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 50).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn resolution_component_flags(&self) -> &BitSlice<u8, Msb0> {
        (&self.data[54..55]).view_bits()
    }

    pub fn end_latitude(&self) -> f64 {
        let raw_value = read_u32_from_bytes(&self.data, 55).unwrap_or(0);
        let value = as_signed!(raw_value, 32, i32) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn end_longitude(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 59).unwrap_or(0) as f64;
        value * (10f64.powf(-6.0))
    }

    pub fn i_direction_increment(&self) -> f64 {
        let value = read_u32_from_bytes(&self.data, 63).unwrap_or(0) as f64;
        let value = value * (10f64.powf(-6.0));

        if self.scanning_mode_flags()[0] == ScanningMode::MinusI {
            value * -1.0
        } else {
            value
        }
    }

    /// N - number of parallels between a pole and the equator
    /// For a full global Gaussian grid, there are 2*N latitudes total
    pub fn n_parallels(&self) -> u32 {
        read_u32_from_bytes(&self.data, 67).unwrap_or(0)
    }

    pub fn scanning_mode_flags(&self) -> ScanningModeFlags {
        ScanningMode::read_flags(self.data[71])
    }

    /// Compute Gaussian latitudes using Newton's method to find roots of Legendre polynomials
    pub fn gaussian_latitudes(&self) -> Vec<f64> {
        let n = self.n_parallels() as usize;
        let nlat = 2 * n;

        // Compute Gaussian latitudes (roots of Legendre polynomial)
        let mut latitudes = compute_gaussian_latitudes(nlat);

        // Check scanning direction and reverse if necessary
        if self.scanning_mode_flags()[1] == ScanningMode::MinusJ {
            // Data starts from north (positive latitudes first)
            // latitudes are already in descending order from compute_gaussian_latitudes
        } else {
            // Data starts from south (negative latitudes first)
            latitudes.reverse();
        }

        latitudes
    }

    pub fn longitudes(&self) -> Vec<f64> {
        let longitude_start = self.start_longitude();
        let longitude_step = self.i_direction_increment();
        (0..self.x_count())
            .map(|i| {
                let mut lon = longitude_start + i as f64 * longitude_step;
                // Normalize to 0..360 range for grids that wrap around the globe
                if lon >= 360.0 {
                    lon -= 360.0;
                } else if lon < 0.0 && longitude_start >= 0.0 {
                    lon += 360.0;
                }
                lon
            })
            .collect()
    }

    pub fn grid_bounds(&self) -> ((f64, f64), (f64, f64)) {
        (
            (self.start_latitude(), self.start_longitude()),
            (self.end_latitude(), self.end_longitude()),
        )
    }
}

/// Compute Gaussian latitudes using Newton-Raphson iteration
/// Returns latitudes in degrees, sorted from north to south (90 to -90)
fn compute_gaussian_latitudes(nlat: usize) -> Vec<f64> {
    let mut latitudes = Vec::with_capacity(nlat);

    // We only need to compute half the roots due to symmetry
    let n_half = nlat / 2;

    for i in 0..n_half {
        // Initial guess using asymptotic approximation
        let theta = PI * (4.0 * (i + 1) as f64 - 1.0) / (4.0 * nlat as f64 + 2.0);
        let mut x = theta.cos();

        // Newton-Raphson iteration to find roots of Legendre polynomial
        for _ in 0..100 {
            let (pn, dpn) = legendre_pn_and_derivative(nlat, x);
            let dx = pn / dpn;
            x -= dx;
            if dx.abs() < 1e-15 {
                break;
            }
        }

        // Convert from cos(theta) to latitude in degrees
        let lat = x.asin() * 180.0 / PI;

        // Add positive latitude (northern hemisphere)
        latitudes.push(lat);
    }

    // Sort northern hemisphere latitudes from highest to lowest
    latitudes.sort_by(|a, b| b.partial_cmp(a).unwrap());

    // Create full latitude array with symmetry
    let mut full_latitudes = Vec::with_capacity(nlat);

    // Add northern hemisphere (positive latitudes, descending)
    for &lat in &latitudes {
        full_latitudes.push(lat);
    }

    // Add southern hemisphere (negative latitudes, descending absolute value)
    for &lat in latitudes.iter().rev() {
        full_latitudes.push(-lat);
    }

    full_latitudes
}

/// Compute Legendre polynomial P_n(x) and its derivative P'_n(x)
/// using the recurrence relation
fn legendre_pn_and_derivative(n: usize, x: f64) -> (f64, f64) {
    if n == 0 {
        return (1.0, 0.0);
    }
    if n == 1 {
        return (x, 1.0);
    }

    let mut p0 = 1.0;
    let mut p1 = x;

    for k in 2..=n {
        let pk = ((2 * k - 1) as f64 * x * p1 - (k - 1) as f64 * p0) / k as f64;
        p0 = p1;
        p1 = pk;
    }

    // Derivative: P'_n(x) = n * (x * P_n(x) - P_{n-1}(x)) / (x^2 - 1)
    let dpn = n as f64 * (x * p1 - p0) / (x * x - 1.0);

    (p1, dpn)
}

impl GridDefinitionTemplate for GaussianTemplate {
    fn proj_name(&self) -> String {
        "latlon".to_string()
    }

    fn proj_params(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        params.insert("a".to_string(), 6367470.0);
        params.insert("b".to_string(), 6367470.0);
        params
    }

    fn proj_string(&self) -> String {
        "+proj=latlon +a=6367470 +b=6367470".to_string()
    }

    fn crs(&self) -> String {
        "EPSG:4326".to_string()
    }

    fn grid_point_count(&self) -> usize {
        (self.parallel_point_count() * self.meridian_point_count()) as usize
    }

    fn is_regular_grid(&self) -> bool {
        // Gaussian grids have regular longitudes but irregular latitudes
        false
    }

    fn y_count(&self) -> usize {
        self.meridian_point_count() as usize
    }

    fn x_count(&self) -> usize {
        self.parallel_point_count() as usize
    }

    fn projector(&self) -> LatLngProjection {
        let latitudes = self.gaussian_latitudes();
        let lat_iter = IrregularCoordinateIterator::new(latitudes);

        let lon_iter = RegularCoordinateIterator::new(
            self.start_longitude(),
            self.i_direction_increment(),
            self.x_count(),
        );

        LatLngProjection::Gaussian(GaussianProjection {
            latitudes: lat_iter,
            longitudes: lon_iter,
            projection_name: self.proj_name(),
            projection_params: self.proj_params(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_latitudes_symmetry() {
        // Test that Gaussian latitudes are symmetric about equator
        let lats = compute_gaussian_latitudes(8);
        assert_eq!(lats.len(), 8);

        // Check symmetry
        let n = lats.len();
        for i in 0..n / 2 {
            assert!(
                (lats[i] + lats[n - 1 - i]).abs() < 1e-10,
                "Latitudes should be symmetric: {} vs {}",
                lats[i],
                lats[n - 1 - i]
            );
        }
    }

    #[test]
    fn test_gaussian_latitudes_range() {
        // Test that latitudes are within valid range
        let lats = compute_gaussian_latitudes(1536);
        assert_eq!(lats.len(), 1536);

        for lat in &lats {
            assert!(
                *lat >= -90.0 && *lat <= 90.0,
                "Latitude {} out of range",
                lat
            );
        }

        // First latitude should be positive (northern hemisphere)
        assert!(lats[0] > 0.0);
        // Last latitude should be negative (southern hemisphere)
        assert!(lats[lats.len() - 1] < 0.0);
    }

    #[test]
    fn test_gaussian_latitudes_n768() {
        // Test N=768 Gaussian grid (like GDAS)
        let lats = compute_gaussian_latitudes(1536);

        // The first latitude for N=768 should be approximately 89.910°
        assert!(
            (lats[0] - 89.91).abs() < 0.01,
            "First latitude should be ~89.91°, got {}",
            lats[0]
        );

        // The last latitude should be approximately -89.910°
        assert!(
            (lats[1535] + 89.91).abs() < 0.01,
            "Last latitude should be ~-89.91°, got {}",
            lats[1535]
        );
    }
}
