"""End-to-end north_up coverage over the 2x2 of {south-first, north-first}
real GRIB fixtures x {north_up off, on}, driving the public decode API the way
a caller does (`parse_grib_array` + `parse_grib_message_metadata(...).latlng`).

HRRR (Lambert) is encoded south-first, so north_up must flip it; GEFS (global
lat/lon) is encoded north-first, so north_up must leave it untouched.
"""

from pathlib import Path

import numpy as np
import pytest

import gribberish as g

TEST_DATA = Path(__file__).resolve().parents[2] / "test-data"

# (fixture, expected native orientation). One decoded message each.
SOUTH_FIRST = "hrrr.t06z.wrfsfcf01-TMP.grib2"
NORTH_FIRST = "geavg.t12z.pgrb2a.0p50.f000"


def _decode(fname, north_up):
    """Return (data, lat, lng) for one message: data is ny x nx; lat/lng are the
    2-D fields for a projected grid or the 1-D axes for a regular grid."""
    raw = (TEST_DATA / fname).read_bytes()
    meta = g.parse_grib_message_metadata(raw, 0)
    ny, nx = meta.grid_shape
    data = np.asarray(g.parse_grib_array(raw, 0, False, north_up)).reshape(ny, nx)
    lat, lng = meta.latlng(False, north_up)
    lat, lng = np.asarray(lat), np.asarray(lng)
    if lat.size == ny * nx:  # projected grid: flattened 2-D fields
        lat, lng = lat.reshape(ny, nx), lng.reshape(ny, nx)
    return data, lat, lng


def _row_lat(lat):
    """First latitude of each row, for either a 2-D field or a 1-D row axis."""
    return lat[:, 0] if lat.ndim == 2 else lat


@pytest.mark.parametrize("north_up", [False, True])
def test_south_first_source_flips_only_when_requested(north_up):
    native_data, native_lat, native_lng = _decode(SOUTH_FIRST, north_up=False)
    native_row_lat = _row_lat(native_lat)
    assert native_row_lat[0] < native_row_lat[-1], "HRRR fixture expected south-first"

    data, lat, lng = _decode(SOUTH_FIRST, north_up=north_up)
    if north_up:
        # Whole rows reversed: row 0 is now the northern-most, and data and
        # both (2-D) coordinate fields moved together.
        assert _row_lat(lat)[0] > _row_lat(lat)[-1]
        np.testing.assert_array_equal(data, native_data[::-1, :])
        np.testing.assert_array_equal(lat, native_lat[::-1, :])
        np.testing.assert_array_equal(lng, native_lng[::-1, :])
    else:
        np.testing.assert_array_equal(data, native_data)


@pytest.mark.parametrize("north_up", [False, True])
def test_north_first_source_is_untouched(north_up):
    native_data, native_lat, _ = _decode(NORTH_FIRST, north_up=False)
    native_row_lat = _row_lat(native_lat)
    assert native_row_lat[0] > native_row_lat[-1], "GEFS fixture expected north-first"

    data, lat, _ = _decode(NORTH_FIRST, north_up=north_up)
    # Already north-first, so north_up is a no-op in both states.
    np.testing.assert_array_equal(data, native_data)
    np.testing.assert_array_equal(lat, native_lat)
