//! Polar-aspect stereographic, the projection of GRIB2 grid definition
//! template 3.20.
//!
//! Reference: Snyder, *Map Projections — A Working Manual*, USGS Professional
//! Paper 1395, Chapter 21 (Stereographic), polar aspect, ellipsoidal form. The
//! conformal-latitude auxiliary `t` (eq. 15-9) and its inverse (eq. 7-9) are
//! cited precisely in `conformal.rs`; this module additionally uses `m_c`
//! (eq. 14-15) and the polar-aspect radius and its inverse. The spherical
//! case is not special-cased: with zero eccentricity the standard parallel
//! form `rho = a * m_c * t / t_c` reduces exactly to the familiar
//! `rho = 2 * a * k0 * tan(pi/4 - lat/2)` with `k0 = (1 + sin(lat_ts)) / 2`.

use mappers::{Ellipsoid, Projection};

use super::conformal::{conformal_t, inverse_conformal_t};
use super::wrap_longitude_degrees;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct PolarStereographic {
    /// Orientation of the grid (`LoV`) in radians: the meridian that runs
    /// straight down the map from the pole.
    lon_origin: f64,
    /// Whether the south pole is on the projection plane.
    south: bool,
    /// `a * m_c / t_c` (or its `lat_ts = +-90` closed form) — everything in
    /// the radius that does not depend on the point being projected.
    rho_scale: f64,
    /// Ellipsoid eccentricity; zero for a spherical earth.
    e: f64,
}

impl PolarStereographic {
    /// `lat_ts_deg` is the latitude of true scale (`LaD` in template 3.20); its
    /// sign is ignored, the hemisphere comes from `south`.
    pub fn new(lon_origin_deg: f64, lat_ts_deg: f64, south: bool, ellipsoid: Ellipsoid) -> Self {
        let e = ellipsoid.E;
        let lat_ts = lat_ts_deg.abs().to_radians();
        let t_c = conformal_t(lat_ts, e);

        // At `lat_ts = +-90` (PROJ's "variant A": scale factor k0 = 1 at the
        // pole) `t_c` is exactly zero and `m_c / t_c` is 0/0, so fall back to
        // the closed-form polar-aspect scale (Snyder eq. 21-33 with k0 = 1)
        // instead of the standard-parallel form below.
        let rho_scale = if t_c == 0.0 {
            2.0 * ellipsoid.A / ((1.0 + e).powf(1.0 + e) * (1.0 - e).powf(1.0 - e)).sqrt()
        } else {
            let m_c = lat_ts.cos() / (1.0 - e * e * lat_ts.sin().powi(2)).sqrt();
            ellipsoid.A * m_c / t_c
        };

        Self {
            lon_origin: lon_origin_deg.to_radians(),
            south,
            rho_scale,
            e,
        }
    }

    /// The southern aspect is the northern one reflected through the equator,
    /// so the formulas below work in the northern frame and flip on the way in
    /// and out.
    fn hemisphere_sign(&self) -> f64 {
        if self.south {
            -1.0
        } else {
            1.0
        }
    }
}

impl Projection for PolarStereographic {
    fn project_unchecked(&self, lon: f64, lat: f64) -> (f64, f64) {
        let sign = self.hemisphere_sign();
        let rho = self.rho_scale * conformal_t(sign * lat.to_radians(), self.e);
        let delta_lon = lon.to_radians() - self.lon_origin;
        (rho * delta_lon.sin(), -sign * rho * delta_lon.cos())
    }

    fn inverse_project_unchecked(&self, x: f64, y: f64) -> (f64, f64) {
        let sign = self.hemisphere_sign();
        let rho = x.hypot(y);
        let lat = sign * inverse_conformal_t(rho / self.rho_scale, self.e);
        let lon = self.lon_origin + x.atan2(-sign * y);
        (wrap_longitude_degrees(lon.to_degrees()), lat.to_degrees())
    }
}

#[cfg(test)]
mod tests {
    use super::PolarStereographic;
    use mappers::{Ellipsoid, Projection};

    fn ncep_sphere() -> Ellipsoid {
        Ellipsoid {
            A: 6_371_229.0,
            B: 6_371_229.0,
            E: 0.0,
            F: 0.0,
        }
    }

    /// NCEP grid 198: north polar, LoV 210, LaD 60, 5953 m square. The corners
    /// are eccodes' values for the AK NAQFC domain, wrapped to [-180, 180).
    #[test]
    fn spherical_corners_match_eccodes_grid_198() {
        let projection = PolarStereographic::new(-150.0, 60.0, false, ncep_sphere());
        let (x0, y0) = projection.project(-178.571, 40.53).unwrap();

        for &(dx, dy, lat, lon) in &[
            (0.0, 0.0, 40.530_000_000_0, -178.571_000_000_0),
            (1.0, 0.0, 40.552_618_942_8, -178.516_280_574_2),
            (824.0, 0.0, 41.739_954_898_4, -124.581_815_826_1),
            (0.0, 1.0, 40.571_569_813_8, -178.600_811_053_4),
            (824.0, 552.0, 63.976_300_235_3, -93.692_321_537_1),
        ] {
            let (got_lon, got_lat) = projection
                .inverse_project(x0 + 5953.0 * dx, y0 + 5953.0 * dy)
                .unwrap();
            assert!(
                (got_lat - lat).abs() < 1e-9,
                "dx={dx} dy={dy} lat {got_lat}"
            );
            assert!(
                (got_lon - lon).abs() < 1e-9,
                "dx={dx} dy={dy} lon {got_lon}"
            );
        }
    }

    /// The north pole is the origin, and the standard parallel is where the
    /// spherical scale factor (1 + sin lat_ts)/2 makes distances true.
    #[test]
    fn north_pole_is_the_origin() {
        let projection = PolarStereographic::new(-150.0, 60.0, false, ncep_sphere());
        let (x, y) = projection.project(0.0, 90.0).unwrap();
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6, "({x}, {y})");
    }

    /// The southern aspect mirrors the northern one: the same grid geometry
    /// about the south pole, which is what projectionCentreFlag bit 1 selects.
    #[test]
    fn southern_aspect_mirrors_the_northern() {
        let north = PolarStereographic::new(0.0, 60.0, false, ncep_sphere());
        let south = PolarStereographic::new(0.0, -60.0, true, ncep_sphere());

        let (xs, ys) = south.project(0.0, -90.0).unwrap();
        assert!(
            xs.abs() < 1e-6 && ys.abs() < 1e-6,
            "south pole at ({xs}, {ys})"
        );

        // A point at -70S under the southern aspect is the mirror of +70N under
        // the northern one: same easting, opposite northing.
        let (xn, yn) = north.project(30.0, 70.0).unwrap();
        let (xs, ys) = south.project(30.0, -70.0).unwrap();
        assert!((xs - xn).abs() < 1e-6, "x {xn} vs {xs}");
        assert!((ys + yn).abs() < 1e-6, "y {yn} vs {ys}");
    }

    /// Round-trip on an ellipsoid, where the inverse has no closed form.
    #[test]
    fn ellipsoidal_round_trip() {
        for &south in &[false, true] {
            let lat_ts = if south { -70.0 } else { 70.0 };
            let projection = PolarStereographic::new(-45.0, lat_ts, south, Ellipsoid::WGS84);
            let sign = if south { -1.0 } else { 1.0 };
            for &(lon, lat) in &[(-45.0, 60.0), (0.0, 75.0), (120.0, 82.5), (-179.0, 55.0)] {
                let lat = sign * lat;
                let (x, y) = projection.project(lon, lat).unwrap();
                let (rlon, rlat) = projection.inverse_project(x, y).unwrap();
                assert!(
                    (rlat - lat).abs() < 1e-9,
                    "south={south} lat {lat} -> {rlat}"
                );
                assert!(
                    (rlon - lon).abs() < 1e-9,
                    "south={south} lon {lon} -> {rlon}"
                );
            }
        }
    }

    /// PROJ ground truth (pyproj) for the forward direction only, independent
    /// of `inverse_project`. A scale or constant-offset error present
    /// consistently in both directions would otherwise pass every test above.
    ///
    /// `+proj=stere +lat_0=90 +lat_ts=60 +lon_0=-150 +a=6371229 +b=6371229`
    #[test]
    fn forward_matches_proj_north_sphere() {
        let projection = PolarStereographic::new(-150.0, 60.0, false, ncep_sphere());

        let (x, y) = projection.project(-178.571, 40.53).unwrap();
        assert!((x - (-2619395.9687247723)).abs() < 1e-6, "x {x}");
        assert!((y - (-4810103.250943904)).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(0.0, 90.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - 0.0).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(-150.0, 60.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - (-3185614.500000001)).abs() < 1e-6, "y {y}");
    }

    /// `+proj=stere +lat_0=-90 +lat_ts=-60 +lon_0=0 +a=6371229 +b=6371229`
    #[test]
    fn forward_matches_proj_south_sphere() {
        let projection = PolarStereographic::new(0.0, -60.0, true, ncep_sphere());

        let (x, y) = projection.project(0.0, -90.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - 0.0).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(30.0, -70.0).unwrap();
        assert!((x - 1048164.7311374075).abs() < 1e-6, "x {x}");
        assert!((y - 1815474.5690317622).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(-45.0, -55.0).unwrap();
        assert!((x - (-2650623.668961142)).abs() < 1e-6, "x {x}");
        assert!((y - 2650623.6689611427).abs() < 1e-6, "y {y}");
    }

    /// `+proj=stere +lat_0=90 +lat_ts=70 +lon_0=-45 +ellps=WGS84`
    ///
    /// The only check that the ellipsoidal path produces correct absolute
    /// positions rather than merely self-consistent ones.
    #[test]
    fn forward_matches_proj_north_wgs84() {
        let projection = PolarStereographic::new(-45.0, 70.0, false, Ellipsoid::WGS84);

        let (x, y) = projection.project(-45.0, 60.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - (-3323160.2706410023)).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(0.0, 75.0).unwrap();
        assert!((x - 1155327.2723032606).abs() < 1e-6, "x {x}");
        assert!((y - (-1155327.2723032609)).abs() < 1e-6, "y {y}");
    }

    /// `lat_ts = +-90` is legal GRIB2 (`LaD` is allowed to be a pole) and is
    /// PROJ's "variant A" form, where the standard-parallel formula's `t_c`
    /// term is exactly zero. This must not produce an infinite `rho_scale`.
    ///
    /// Ground truth is PROJ (pyproj) output for the proj4 strings noted below,
    /// taken verbatim.
    #[test]
    fn forward_matches_proj_at_the_pole_variant_a() {
        // +proj=stere +lat_0=90 +lat_ts=90 +lon_0=-150 +a=6371229 +b=6371229
        let projection = PolarStereographic::new(-150.0, 90.0, false, ncep_sphere());

        let (x, y) = projection.project(-178.571, 40.53).unwrap();
        assert!((x - (-2807460.1379085644)).abs() < 1e-6, "x {x}");
        assert!((y - (-5155453.126402948)).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(0.0, 90.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - 0.0).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(-150.0, 60.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - (-3414331.3306874996)).abs() < 1e-6, "y {y}");

        // +proj=stere +lat_0=90 +lat_ts=90 +lon_0=0 +ellps=WGS84
        let projection = PolarStereographic::new(0.0, 90.0, false, Ellipsoid::WGS84);

        let (x, y) = projection.project(0.0, 60.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - (-3426439.3534922632)).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(45.0, 75.0).unwrap();
        assert!((x - 1191233.1965918639).abs() < 1e-6, "x {x}");
        assert!((y - (-1191233.1965918639)).abs() < 1e-6, "y {y}");

        // +proj=stere +lat_0=-90 +lat_ts=-90 +lon_0=0 +a=6371229 +b=6371229
        let projection = PolarStereographic::new(0.0, -90.0, true, ncep_sphere());

        let (x, y) = projection.project(0.0, -90.0).unwrap();
        assert!((x - 0.0).abs() < 1e-6, "x {x}");
        assert!((y - 0.0).abs() < 1e-6, "y {y}");

        let (x, y) = projection.project(30.0, -70.0).unwrap();
        assert!((x - 1123419.5729722127).abs() < 1e-6, "x {x}");
        assert!((y - 1945819.7786052045).abs() < 1e-6, "y {y}");
    }

    /// The inverse must also stay finite and recover the original point at
    /// `lat_ts = 90`, not just the forward direction.
    #[test]
    fn round_trip_at_lat_ts_90() {
        for &south in &[false, true] {
            let lat_ts = if south { -90.0 } else { 90.0 };
            let projection = PolarStereographic::new(-45.0, lat_ts, south, Ellipsoid::WGS84);
            let sign = if south { -1.0 } else { 1.0 };
            for &(lon, lat) in &[(-45.0, 60.0), (0.0, 75.0), (120.0, 82.5), (-179.0, 55.0)] {
                let lat = sign * lat;
                let (x, y) = projection.project(lon, lat).unwrap();
                assert!(x.is_finite() && y.is_finite(), "south={south} ({x}, {y})");
                let (rlon, rlat) = projection.inverse_project(x, y).unwrap();
                assert!(
                    (rlat - lat).abs() < 1e-9,
                    "south={south} lat {lat} -> {rlat}"
                );
                assert!(
                    (rlon - lon).abs() < 1e-9,
                    "south={south} lon {lon} -> {rlon}"
                );
            }
        }
    }
}
