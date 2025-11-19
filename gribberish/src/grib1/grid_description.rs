/// GRIB1 Grid Description Section (Section 2)
///
/// The GDS describes the grid geometry and projection.
/// This implementation focuses on lat/lon grids (type 0) initially.

use crate::templates::grid_definition::grid_definition_template::GridDefinitionTemplate;
use crate::utils::convert::read_u16_from_bytes;
use crate::utils::iter::projection::{LatLngProjection, PlateCareeProjection, RegularCoordinateIterator};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Grib1Grid {
    LatLon(LatLonGrid),
    Gaussian(GaussianGrid),
    // Other grid types can be added later
    Unsupported { grid_type: u8 },
}

#[derive(Debug, Clone)]
pub struct LatLonGrid {
    pub ni: usize,              // Number of points along x-axis
    pub nj: usize,              // Number of points along y-axis
    pub lat1: f64,              // Latitude of first grid point (degrees)
    pub lon1: f64,              // Longitude of first grid point (degrees)
    pub lat2: f64,              // Latitude of last grid point (degrees)
    pub lon2: f64,              // Longitude of last grid point (degrees)
    pub di: f64,                // i-direction increment (degrees)
    pub dj: f64,                // j-direction increment (degrees)
    pub scanning_mode: u8,      // Scanning mode flags
}

#[derive(Debug, Clone)]
pub struct GaussianGrid {
    pub ni: usize,
    pub nj: usize,
    pub lat1: f64,
    pub lon1: f64,
    pub lat2: f64,
    pub lon2: f64,
    pub di: f64,
    pub n: u16,                 // Number of latitude circles between pole and equator
    pub scanning_mode: u8,
}

impl Grib1Grid {
    pub fn from_data(data: &[u8]) -> Result<Self, String> {
        if data.len() < 6 {
            return Err("GDS too short".to_string());
        }

        let grid_type = data[5];

        match grid_type {
            0 => Self::parse_latlon(data),
            4 => Self::parse_gaussian(data),
            _ => Ok(Grib1Grid::Unsupported { grid_type }),
        }
    }

    fn parse_latlon(data: &[u8]) -> Result<Self, String> {
        if data.len() < 32 {
            return Err("Lat/Lon GDS too short".to_string());
        }

        let ni = read_u16_from_bytes(data, 6).ok_or("Failed to read ni")? as usize;
        let nj = read_u16_from_bytes(data, 8).ok_or("Failed to read nj")? as usize;

        // Coordinates are stored as signed 24-bit integers in millidegrees
        let lat1 = read_signed_24(data, 10) as f64 / 1000.0;
        let lon1 = read_signed_24(data, 13) as f64 / 1000.0;

        let _resolution_flags = data[16];

        let lat2 = read_signed_24(data, 17) as f64 / 1000.0;
        let lon2 = read_signed_24(data, 20) as f64 / 1000.0;

        let di = read_u16_from_bytes(data, 23).ok_or("Failed to read di")? as f64 / 1000.0;
        let dj = read_u16_from_bytes(data, 25).ok_or("Failed to read dj")? as f64 / 1000.0;

        let scanning_mode = data[27];

        Ok(Grib1Grid::LatLon(LatLonGrid {
            ni,
            nj,
            lat1,
            lon1,
            lat2,
            lon2,
            di,
            dj,
            scanning_mode,
        }))
    }

    fn parse_gaussian(data: &[u8]) -> Result<Self, String> {
        if data.len() < 32 {
            return Err("Gaussian GDS too short".to_string());
        }

        let ni = read_u16_from_bytes(data, 6).ok_or("Failed to read ni")? as usize;
        let nj = read_u16_from_bytes(data, 8).ok_or("Failed to read nj")? as usize;

        let lat1 = read_signed_24(data, 10) as f64 / 1000.0;
        let lon1 = read_signed_24(data, 13) as f64 / 1000.0;

        let lat2 = read_signed_24(data, 17) as f64 / 1000.0;
        let lon2 = read_signed_24(data, 20) as f64 / 1000.0;

        let di = read_u16_from_bytes(data, 23).ok_or("Failed to read di")? as f64 / 1000.0;
        let n = read_u16_from_bytes(data, 25).ok_or("Failed to read n")?;

        let scanning_mode = data[27];

        Ok(Grib1Grid::Gaussian(GaussianGrid {
            ni,
            nj,
            lat1,
            lon1,
            lat2,
            lon2,
            di,
            n,
            scanning_mode,
        }))
    }

    /// Get grid dimensions (ni, nj)
    pub fn dimensions(&self) -> (usize, usize) {
        match self {
            Grib1Grid::LatLon(grid) => (grid.ni, grid.nj),
            Grib1Grid::Gaussian(grid) => (grid.ni, grid.nj),
            Grib1Grid::Unsupported { .. } => (0, 0),
        }
    }

    /// Get total number of grid points
    pub fn num_points(&self) -> usize {
        let (ni, nj) = self.dimensions();
        ni * nj
    }

    /// Check if grid scans in +i direction
    pub fn scans_positively_i(&self) -> bool {
        let mode = match self {
            Grib1Grid::LatLon(g) => g.scanning_mode,
            Grib1Grid::Gaussian(g) => g.scanning_mode,
            _ => 0,
        };
        (mode & 0x80) == 0
    }

    /// Check if grid scans in -j direction
    pub fn scans_negatively_j(&self) -> bool {
        let mode = match self {
            Grib1Grid::LatLon(g) => g.scanning_mode,
            Grib1Grid::Gaussian(g) => g.scanning_mode,
            _ => 0,
        };
        (mode & 0x40) != 0
    }

    /// Generate latitude values for the grid
    pub fn latitudes(&self) -> Vec<f64> {
        match self {
            Grib1Grid::LatLon(grid) => {
                let mut lats = Vec::with_capacity(grid.nj);
                for j in 0..grid.nj {
                    let lat = if self.scans_negatively_j() {
                        grid.lat1 - (j as f64) * grid.dj
                    } else {
                        grid.lat1 + (j as f64) * grid.dj
                    };
                    lats.push(lat);
                }
                lats
            }
            Grib1Grid::Gaussian(_) => {
                // Gaussian grid latitudes require special calculation
                // For now, return empty - will implement later if needed
                vec![]
            }
            Grib1Grid::Unsupported { .. } => vec![],
        }
    }

    /// Generate longitude values for the grid
    pub fn longitudes(&self) -> Vec<f64> {
        match self {
            Grib1Grid::LatLon(grid) => {
                let mut lons = Vec::with_capacity(grid.ni);
                for i in 0..grid.ni {
                    let mut lon = if self.scans_positively_i() {
                        grid.lon1 + (i as f64) * grid.di
                    } else {
                        grid.lon1 - (i as f64) * grid.di
                    };

                    // Normalize to -180..180 or 0..360 depending on input
                    if lon > 360.0 {
                        lon -= 360.0;
                    } else if lon < 0.0 && grid.lon1 >= 0.0 {
                        lon += 360.0;
                    }

                    lons.push(lon);
                }
                lons
            }
            Grib1Grid::Gaussian(grid) => {
                let mut lons = Vec::with_capacity(grid.ni);
                for i in 0..grid.ni {
                    let mut lon = if self.scans_positively_i() {
                        grid.lon1 + (i as f64) * grid.di
                    } else {
                        grid.lon1 - (i as f64) * grid.di
                    };

                    if lon > 360.0 {
                        lon -= 360.0;
                    } else if lon < 0.0 && grid.lon1 >= 0.0 {
                        lon += 360.0;
                    }

                    lons.push(lon);
                }
                lons
            }
            Grib1Grid::Unsupported { .. } => vec![],
        }
    }
}

/// Read a signed 24-bit integer from 3 bytes (big-endian)
fn read_signed_24(data: &[u8], offset: usize) -> i32 {
    if offset + 3 > data.len() {
        return 0;
    }

    let unsigned = ((data[offset] as u32) << 16)
        | ((data[offset + 1] as u32) << 8)
        | (data[offset + 2] as u32);

    // Convert to signed: if bit 23 is set, it's negative
    if unsigned & 0x800000 != 0 {
        // Sign-extend to 32 bits
        (unsigned | 0xFF000000) as i32
    } else {
        unsigned as i32
    }
}

// Implement GridDefinitionTemplate trait for GRIB1 grids to enable xarray backend support
impl GridDefinitionTemplate for Grib1Grid {
    fn proj_name(&self) -> String {
        match self {
            Grib1Grid::LatLon(_) => "latlon".to_string(),
            Grib1Grid::Gaussian(_) => "latlon".to_string(), // Gaussian grids are also on lat/lon
            Grib1Grid::Unsupported { .. } => "unknown".to_string(),
        }
    }

    fn proj_params(&self) -> HashMap<String, f64> {
        let mut params = HashMap::new();
        // Use WGS84 ellipsoid parameters (common default)
        params.insert("a".to_string(), 6378137.0);
        params.insert("b".to_string(), 6356752.314245);
        params
    }

    fn proj_string(&self) -> String {
        match self {
            Grib1Grid::LatLon(_) | Grib1Grid::Gaussian(_) => {
                "+proj=latlon +a=6378137 +b=6356752.314245".to_string()
            }
            Grib1Grid::Unsupported { .. } => "+proj=latlon".to_string(),
        }
    }

    fn crs(&self) -> String {
        match self {
            Grib1Grid::LatLon(_) | Grib1Grid::Gaussian(_) => "EPSG:4326".to_string(),
            Grib1Grid::Unsupported { .. } => "unknown".to_string(),
        }
    }

    fn grid_point_count(&self) -> usize {
        self.num_points()
    }

    fn is_regular_grid(&self) -> bool {
        match self {
            Grib1Grid::LatLon(_) => true,
            Grib1Grid::Gaussian(_) => false, // Gaussian grids are not regular in latitude
            Grib1Grid::Unsupported { .. } => false,
        }
    }

    fn y_count(&self) -> usize {
        let (_, nj) = self.dimensions();
        nj
    }

    fn x_count(&self) -> usize {
        let (ni, _) = self.dimensions();
        ni
    }

    fn projector(&self) -> LatLngProjection {
        match self {
            Grib1Grid::LatLon(grid) => {
                // Determine the direction increment based on scanning mode
                let lat_start = if self.scans_negatively_j() {
                    grid.lat1
                } else {
                    grid.lat1
                };

                let lat_increment = if self.scans_negatively_j() {
                    -grid.dj
                } else {
                    grid.dj
                };

                let lon_start = if self.scans_positively_i() {
                    grid.lon1
                } else {
                    grid.lon1
                };

                let lon_increment = if self.scans_positively_i() {
                    grid.di
                } else {
                    -grid.di
                };

                let lat_iter = RegularCoordinateIterator::new(
                    lat_start,
                    lat_increment,
                    grid.nj,
                );

                let lon_iter = RegularCoordinateIterator::new(
                    lon_start,
                    lon_increment,
                    grid.ni,
                );

                LatLngProjection::PlateCaree(PlateCareeProjection {
                    latitudes: lat_iter,
                    longitudes: lon_iter,
                    projection_name: self.proj_name(),
                    projection_params: self.proj_params(),
                })
            }
            Grib1Grid::Gaussian(grid) => {
                // For Gaussian grids, use a simple regular projection
                // Note: This is a simplification; proper Gaussian grid support would
                // require computing actual Gaussian latitudes
                let lon_start = if self.scans_positively_i() {
                    grid.lon1
                } else {
                    grid.lon1
                };

                let lon_increment = if self.scans_positively_i() {
                    grid.di
                } else {
                    -grid.di
                };

                // For latitudes, we'll use a simple linear interpolation
                // between lat1 and lat2 as a fallback
                let lat_increment = if grid.nj > 1 {
                    (grid.lat2 - grid.lat1) / (grid.nj - 1) as f64
                } else {
                    0.0
                };

                let lat_iter = RegularCoordinateIterator::new(
                    grid.lat1,
                    lat_increment,
                    grid.nj,
                );

                let lon_iter = RegularCoordinateIterator::new(
                    lon_start,
                    lon_increment,
                    grid.ni,
                );

                LatLngProjection::PlateCaree(PlateCareeProjection {
                    latitudes: lat_iter,
                    longitudes: lon_iter,
                    projection_name: self.proj_name(),
                    projection_params: self.proj_params(),
                })
            }
            Grib1Grid::Unsupported { .. } => {
                // Return an empty projection
                let lat_iter = RegularCoordinateIterator::new(0.0, 1.0, 0);
                let lon_iter = RegularCoordinateIterator::new(0.0, 1.0, 0);

                LatLngProjection::PlateCaree(PlateCareeProjection {
                    latitudes: lat_iter,
                    longitudes: lon_iter,
                    projection_name: self.proj_name(),
                    projection_params: self.proj_params(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_signed_24() {
        // Positive value
        let data = [0x00, 0x01, 0x00]; // 256
        assert_eq!(read_signed_24(&data, 0), 256);

        // Negative value
        let data = [0xFF, 0xFF, 0xFF]; // -1
        assert_eq!(read_signed_24(&data, 0), -1);
    }

    #[test]
    fn test_latlon_grid_basics() {
        // Create a simple 2x2 lat/lon grid
        let mut data = vec![0u8; 32];
        data[5] = 0;  // Grid type 0 (lat/lon)
        data[6..8].copy_from_slice(&2u16.to_be_bytes());  // ni = 2
        data[8..10].copy_from_slice(&2u16.to_be_bytes()); // nj = 2

        // lat1 = 90.0 degrees (90000 millidegrees)
        data[10..13].copy_from_slice(&[0x01, 0x5F, 0x90]);
        // lon1 = 0.0 degrees
        data[13..16].copy_from_slice(&[0x00, 0x00, 0x00]);

        // lat2 = 89.0 degrees
        data[17..20].copy_from_slice(&[0x01, 0x5B, 0x98]);
        // lon2 = 1.0 degrees (1000 millidegrees)
        data[20..23].copy_from_slice(&[0x00, 0x03, 0xE8]);

        // di = 0.5 degrees (500 millidegrees)
        data[23..25].copy_from_slice(&500u16.to_be_bytes());
        // dj = 1.0 degrees (1000 millidegrees)
        data[25..27].copy_from_slice(&1000u16.to_be_bytes());

        data[27] = 0x00; // Scanning mode: +i, +j

        let grid = Grib1Grid::from_data(&data).unwrap();

        match grid {
            Grib1Grid::LatLon(g) => {
                assert_eq!(g.ni, 2);
                assert_eq!(g.nj, 2);
                assert!((g.lat1 - 90.0).abs() < 0.1);
                assert!((g.di - 0.5).abs() < 0.01);
            }
            _ => panic!("Expected LatLon grid"),
        }
    }
}
