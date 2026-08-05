//! Normal-aspect Mercator, the projection of GRIB2 grid definition template
//! 3.10.
//!
//! Reference: Snyder, *Map Projections — A Working Manual*, USGS Professional
//! Paper 1395, equations 7-6 through 7-9. The scale factor comes from the
//! template's `LaD`, the latitude at which the encoded grid lengths are true.

use mappers::{Ellipsoid, Projection};

use super::conformal::{conformal_t, inverse_conformal_t};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Mercator {
    /// Central meridian in radians. GRIB2 template 3.10 has no field for this,
    /// so grids built from it always pass zero.
    lon_origin: f64,
    /// Semi-major axis times the scale factor at the standard parallel.
    a_k0: f64,
    /// Ellipsoid eccentricity; zero for a spherical earth.
    e: f64,
}

impl Mercator {
    /// `lat_ts_deg` is the latitude of true scale (`LaD` in template 3.10).
    pub fn new(lon_origin_deg: f64, lat_ts_deg: f64, ellipsoid: Ellipsoid) -> Self {
        let e = ellipsoid.E;
        let lat_ts = lat_ts_deg.to_radians();
        let k0 = lat_ts.cos() / (1.0 - e * e * lat_ts.sin().powi(2)).sqrt();

        Self {
            lon_origin: lon_origin_deg.to_radians(),
            a_k0: ellipsoid.A * k0,
            e,
        }
    }
}

impl Projection for Mercator {
    fn project_unchecked(&self, lon: f64, lat: f64) -> (f64, f64) {
        let x = self.a_k0 * (lon.to_radians() - self.lon_origin);
        // y = -a*k0*ln(t) is the ellipsoidal Mercator northing; on a sphere t
        // is tan(pi/4 - lat/2), so this is the familiar ln(tan(pi/4 + lat/2)).
        let y = -self.a_k0 * conformal_t(lat.to_radians(), self.e).ln();
        (x, y)
    }

    fn inverse_project_unchecked(&self, x: f64, y: f64) -> (f64, f64) {
        let lon = self.lon_origin + x / self.a_k0;
        let lat = inverse_conformal_t((-y / self.a_k0).exp(), self.e);
        (
            super::wrap_longitude_degrees(lon.to_degrees()),
            lat.to_degrees(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Mercator;
    use mappers::{Ellipsoid, Projection};

    /// The sphere NCEP uses for the NAQFC Hawaii grid (shape of earth 6).
    fn ncep_sphere() -> Ellipsoid {
        Ellipsoid {
            A: 6_371_229.0,
            B: 6_371_229.0,
            E: 0.0,
            F: 0.0,
        }
    }

    /// Grid 196 is 2500 m square with true scale at 20N. One grid step east of
    /// the first point must land on the longitude eccodes reports for column 1.
    #[test]
    fn spherical_step_matches_eccodes_grid_196() {
        let projection = Mercator::new(0.0, 20.0, ncep_sphere());
        let (x0, y0) = projection.project(-161.525, 18.073).unwrap();

        let (lon, lat) = projection.inverse_project(x0 + 2500.0, y0).unwrap();
        assert!((lon - -161.501_074_908_4).abs() < 1e-9, "lon {lon}");
        assert!((lat - 18.073).abs() < 1e-9, "lat {lat}");

        // 320 steps east reaches the last column of the grid.
        let (lon, _) = projection.inverse_project(x0 + 2500.0 * 320.0, y0).unwrap();
        assert!((lon - -153.868_970_682_6).abs() < 1e-9, "lon {lon}");

        // 224 steps north reaches the last row; longitude is unchanged, which
        // is the property that distinguishes Mercator from the conic grids.
        let (lon, lat) = projection.inverse_project(x0, y0 + 2500.0 * 224.0).unwrap();
        assert!((lat - 23.088_134_808_7).abs() < 1e-9, "lat {lat}");
        assert!((lon - -161.525).abs() < 1e-9, "lon {lon}");
    }

    /// Round-trip on an ellipsoid, where the inverse has no closed form.
    #[test]
    fn ellipsoidal_round_trip() {
        let projection = Mercator::new(0.0, 20.0, Ellipsoid::WGS84);
        for &(lon, lat) in &[(-161.525, 18.073), (0.0, 0.0), (45.0, -33.5), (179.0, 60.0)] {
            let (x, y) = projection.project(lon, lat).unwrap();
            let (rlon, rlat) = projection.inverse_project(x, y).unwrap();
            assert!((rlon - lon).abs() < 1e-9, "lon {lon} -> {rlon}");
            assert!((rlat - lat).abs() < 1e-9, "lat {lat} -> {rlat}");
        }
    }

    /// True scale at the standard parallel: one metre of easting is one metre
    /// on the ground there, and the equator is y = 0.
    #[test]
    fn standard_parallel_sets_the_scale() {
        let sphere = ncep_sphere();
        let projection = Mercator::new(0.0, 20.0, sphere);
        let (x0, _) = projection.project(0.0, 20.0).unwrap();
        let (x1, _) = projection.project(1.0, 20.0).unwrap();
        let expected = sphere.A * 1f64.to_radians() * 20f64.to_radians().cos();
        assert!((x1 - x0 - expected).abs() < 1e-6);

        let (_, y) = projection.project(0.0, 0.0).unwrap();
        assert!(y.abs() < 1e-9);
    }

    /// External-oracle check: absolute forward (x, y) against PROJ, not
    /// against this crate's own inverse. The other tests only ever read
    /// projected coordinates back through `inverse_project`, so a scale or
    /// constant-offset error present consistently in both directions would
    /// slip past them; this test pins the forward direction independently.
    ///
    /// Expected values are PROJ output for
    /// `+proj=merc +lat_ts=20 +lon_0=0 +a=6371229 +b=6371229` (via pyproj),
    /// not self-generated.
    #[test]
    fn forward_projection_matches_proj_oracle() {
        let projection = Mercator::new(0.0, 20.0, ncep_sphere());
        for &(lon, lat, expected_x, expected_y) in &[
            (
                -161.525,
                18.073,
                -16_878_200.780_530_702,
                1_920_617.792_042_613_5,
            ),
            (0.0, 0.0, 0.0, 0.0),
            (45.0, -33.5, 4_702_176.351_177_102, -3_718_898.598_057_832_6),
        ] {
            let (x, y) = projection.project(lon, lat).unwrap();
            assert!(
                (x - expected_x).abs() < 1e-6,
                "x for ({lon}, {lat}): {x} vs {expected_x}"
            );
            assert!(
                (y - expected_y).abs() < 1e-6,
                "y for ({lon}, {lat}): {y} vs {expected_y}"
            );
        }
    }
}
