//! Conformal-latitude helpers shared by the Mercator and polar stereographic
//! projections.
//!
//! Reference: Snyder, *Map Projections — A Working Manual*, USGS Professional
//! Paper 1395, equations 15-9 and 7-9. Both functions accept an eccentricity of
//! zero, in which case they reduce to the spherical forms, so callers need only
//! one code path for spherical and ellipsoidal earths.

use std::f64::consts::{FRAC_PI_2, FRAC_PI_4};

/// Snyder's `t` (eq. 15-9), the conformal-latitude auxiliary.
///
/// `lat` is in radians and `e` is the ellipsoid eccentricity (not its square).
pub(crate) fn conformal_t(lat: f64, e: f64) -> f64 {
    let tangent = (FRAC_PI_4 - lat / 2.0).tan();
    if e == 0.0 {
        return tangent;
    }
    let sin_lat = lat.sin();
    let ratio = (1.0 - e * sin_lat) / (1.0 + e * sin_lat);
    tangent / ratio.powf(e / 2.0)
}

/// Recover a geodetic latitude in radians from Snyder's `t` (eq. 7-9).
///
/// The ellipsoidal case has no closed form; the standard fixed-point iteration
/// converges in a handful of steps for every terrestrial eccentricity, so the
/// iteration cap is a safety net rather than a normal exit.
pub(crate) fn inverse_conformal_t(t: f64, e: f64) -> f64 {
    let mut lat = FRAC_PI_2 - 2.0 * t.atan();
    if e == 0.0 {
        return lat;
    }
    for _ in 0..15 {
        let sin_lat = lat.sin();
        let ratio = (1.0 - e * sin_lat) / (1.0 + e * sin_lat);
        let next = FRAC_PI_2 - 2.0 * (t * ratio.powf(e / 2.0)).atan();
        if (next - lat).abs() < 1e-14 {
            return next;
        }
        lat = next;
    }
    lat
}

#[cfg(test)]
mod tests {
    use super::{conformal_t, inverse_conformal_t};
    use std::f64::consts::FRAC_PI_4;

    /// On a sphere Snyder's `t` collapses to `tan(pi/4 - lat/2)`.
    #[test]
    fn spherical_t_is_the_tangent_half_angle() {
        let lat = 60f64.to_radians();
        let expected = (FRAC_PI_4 - lat / 2.0).tan();
        assert!((conformal_t(lat, 0.0) - expected).abs() < 1e-15);
        // t = 1 at the equator and 0 at the north pole, for any eccentricity.
        assert!((conformal_t(0.0, 0.0) - 1.0).abs() < 1e-15);
        assert!(conformal_t(90f64.to_radians(), 0.0).abs() < 1e-15);
    }

    /// The inverse must recover the latitude it started from, on the sphere and
    /// on WGS84, where the ellipsoidal correction is largest at mid latitudes.
    #[test]
    fn inverse_round_trips_for_sphere_and_wgs84() {
        for &e in &[0.0, 0.081_819_190_842_622] {
            for &deg in &[-80.0f64, -45.0, -20.0, 0.0, 20.0, 45.0, 60.0, 80.0] {
                let lat = deg.to_radians();
                let recovered = inverse_conformal_t(conformal_t(lat, e), e);
                assert!(
                    (recovered - lat).abs() < 1e-12,
                    "e={e} lat={deg}: got {}",
                    recovered.to_degrees()
                );
            }
        }
    }

    /// The ellipsoidal correction must actually be applied, not silently
    /// dropped — WGS84 and the sphere must disagree away from the equator.
    #[test]
    fn eccentricity_changes_the_result() {
        let lat = 45f64.to_radians();
        let spherical = conformal_t(lat, 0.0);
        let wgs84 = conformal_t(lat, 0.081_819_190_842_622);
        assert!((spherical - wgs84).abs() > 1e-4);
    }
}
