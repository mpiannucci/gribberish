import pytest
from pathlib import Path

xr = pytest.importorskip("xarray")

from gribberish.gribberish_backend import GribberishBackend


def test_xarray_backend_gefs_ensemble():
    """Test reading GEFS ensemble average file using xarray backend"""
    # Open the GEFS ensemble average file using the gribberish backend
    ds = xr.open_dataset(
        "./../gribberish/tests/data/geavg.t12z.pgrb2a.0p50.f000",
        engine="gribberish"
    )

    # Verify dataset was loaded
    assert ds is not None

    # Ensure expected data variables are present (variable names no longer have redundant prefix)
    expected_data_vars = {
        'cape', 'cin', 'hgt_isobar_ens', 'hgt_sfc_ens', 'icetk', 'pres',
        'prmsl', 'pwat', 'rh_hag_ens', 'rh_isobar_ens', 'snod', 'tmp_hag_ens',
        'tmp_isobar_ens', 'tsoil', 'ugrd_hag_ens', 'ugrd_isobar_ens',
        'vgrd_hag_ens', 'vgrd_isobar_ens', 'vvel', 'weasd'
    }
    assert set(ds.data_vars.keys()) == expected_data_vars

    # Verify coordinates are present
    assert 'time' in ds.coords
    assert 'latitude' in ds.coords
    assert 'longitude' in ds.coords

    # Check dimensions of 2D variables
    assert ds.cape.values.shape == (1, 361, 720)

    # Check dimension of 3D variables
    assert ds.tmp_isobar_ens.values.shape == (1, 10, 361, 720)


def test_xarray_backend_era5_grib1():
    """Test reading ERA5 GRIB1 file with levels and members using xarray backend"""
    # Open the ERA5 GRIB1 file using the gribberish backend
    ds = xr.open_dataset(
        "./../gribberish/tests/data/era5-levels-members.grib",
        engine="gribberish"
    )

    # Verify dataset was loaded
    assert ds is not None

    # Check that we have the expected variable (geopotential)
    assert 'z' in ds.variables or 'geopotential' in ds.data_vars.keys()

    # Check coordinates are present
    assert 'latitude' in ds.variables
    assert 'longitude' in ds.variables
    assert 'time' in ds.variables

    # Verify dimensions based on the Rust test expectations (61, 120)
    # The data array should have shape (time, lat, lon) or similar
    geopotential_var = 'z' if 'z' in ds.data_vars else list(ds.data_vars.keys())[0]
    data_shape = ds[geopotential_var].values.shape

    # Check that lat/lon dimensions match expected (61, 120)
    # Note: Order might be (time, lat, lon) so we check the last two dimensions
    assert data_shape[-2:] == (61, 120) or data_shape[-2:] == (120, 61), \
        f"Expected grid dimensions (61, 120) but got {data_shape}"

    # Verify the data can be loaded
    data = ds[geopotential_var].values
    assert data.size > 0

    # Check that the data is not all zeros or NaNs
    import numpy as np
    assert not np.all(data == 0), "Data should not be all zeros"
    assert not np.all(np.isnan(data)), "Data should not be all NaNs"


def test_xarray_backend_can_open_grib1_extensions():
    backend = GribberishBackend()
    assert backend.guess_can_open("example.grib1")
    assert backend.guess_can_open("example.GRIB1")
    assert backend.guess_can_open("example.grib")
    assert backend.guess_can_open("example.grib2")


def test_xarray_backend_ecmwf_soil_tiny_fixture():
    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "test-data" / "ecmwf_soil_8vars_tiny.grib1"
    ds = xr.open_dataset(
        str(fixture),
        engine="gribberish",
    )

    expected = {"swvl1", "swvl2", "swvl3", "swvl4", "stl1", "stl2", "stl3", "stl4"}
    assert expected.issubset(set(ds.data_vars.keys()))
    for var in expected:
        assert ds[var].values.shape[-2:] == (4, 4)


def test_variables_at_same_level_are_separate():
    """Test that multiple variables at the same level type are correctly separated.

    This verifies the fix for variable grouping - variables like TMP, UGRD, VGRD
    at the same level type (e.g., 'hag' or 'isobar') should remain as separate
    data variables with distinct data, not be incorrectly merged together.
    """
    ds = xr.open_dataset(
        "./../gribberish/tests/data/geavg.t12z.pgrb2a.0p50.f000",
        engine="gribberish"
    )

    # Multiple variables should exist at the 'hag' (height above ground) level
    hag_vars = ['tmp_hag_ens', 'ugrd_hag_ens', 'vgrd_hag_ens', 'rh_hag_ens']
    for var in hag_vars:
        assert var in ds.data_vars, f"Expected variable {var} to exist"

    # Multiple variables should exist at the 'isobar' level
    isobar_vars = ['tmp_isobar_ens', 'ugrd_isobar_ens', 'vgrd_isobar_ens',
                   'rh_isobar_ens', 'hgt_isobar_ens']
    for var in isobar_vars:
        assert var in ds.data_vars, f"Expected variable {var} to exist"

    # Verify these are genuinely different variables with different data
    # Temperature and wind components should have different values
    import numpy as np
    tmp_data = ds.tmp_hag_ens.values
    ugrd_data = ds.ugrd_hag_ens.values
    vgrd_data = ds.vgrd_hag_ens.values

    # The data arrays should NOT be identical (they're different physical quantities)
    assert not np.allclose(tmp_data, ugrd_data, equal_nan=True), \
        "TMP and UGRD should have different data"
    assert not np.allclose(tmp_data, vgrd_data, equal_nan=True), \
        "TMP and VGRD should have different data"
    assert not np.allclose(ugrd_data, vgrd_data, equal_nan=True), \
        "UGRD and VGRD should have different data"

    # Each variable should have proper attributes identifying its parameter
    assert 'tmp' in ds.tmp_hag_ens.attrs.get('standard_name', '').lower() or \
           'temperature' in ds.tmp_hag_ens.attrs.get('standard_name', '').lower()
    assert 'wind' in ds.ugrd_hag_ens.attrs.get('standard_name', '').lower() or \
           'u-component' in ds.ugrd_hag_ens.attrs.get('standard_name', '').lower()


def test_variable_naming_without_redundant_prefix():
    """Test that variable names don't have redundant variable prefixes.

    Previously, variables were named like 'tmp_TMPisobar_ens' (variable name
    appeared twice). Now they should be named like 'tmp_isobar_ens'.
    """
    ds = xr.open_dataset(
        "./../gribberish/tests/data/geavg.t12z.pgrb2a.0p50.f000",
        engine="gribberish"
    )

    # Check that no variable names have doubled prefixes like 'TMP' in 'tmp_TMPisobar'
    for var_name in ds.data_vars.keys():
        # Variable names should be lowercase
        assert var_name == var_name.lower(), \
            f"Variable name '{var_name}' should be lowercase"

        # Variable names shouldn't have patterns like 'tmp_TMP' or 'ugrd_UGRD'
        parts = var_name.split('_')
        if len(parts) >= 2:
            # Check the second part isn't just an uppercase version of the first
            assert parts[1].lower() != parts[0], \
                f"Variable name '{var_name}' has redundant prefix"


def test_longitude_normalization_grib2():
    """Test that longitude values are correctly normalized for global GRIB2 grids.

    This validates the fix for longitude wrapping - all longitude values should
    be in a valid range [0, 360) for global grids.
    """
    import numpy as np

    ds = xr.open_dataset(
        "./../gribberish/tests/data/geavg.t12z.pgrb2a.0p50.f000",
        engine="gribberish"
    )

    # Get longitude coordinate
    lons = ds.longitude.values

    # Verify longitude dimensions (720 points at 0.5° resolution)
    assert len(lons) == 720, f"Expected 720 longitude points, got {len(lons)}"

    # Verify longitude range (0 to 359.5 at 0.5° steps)
    assert np.abs(lons[0] - 0.0) < 0.001, f"First longitude should be 0°, got {lons[0]}"
    assert np.abs(lons[-1] - 359.5) < 0.001, f"Last longitude should be 359.5°, got {lons[-1]}"

    # All longitudes should be in valid range [0, 360)
    assert np.all(lons >= 0.0), "All longitudes should be >= 0"
    assert np.all(lons < 360.0), "All longitudes should be < 360"

    # Verify monotonic increase (for grids starting at 0°)
    assert np.all(np.diff(lons) > 0), "Longitudes should be monotonically increasing"


def test_longitude_normalization_grib1():
    """Test that longitude values are correctly normalized for GRIB1 grids.

    This validates the fix for longitude wrapping on GRIB1 files (ERA5).
    """
    import numpy as np

    ds = xr.open_dataset(
        "./../gribberish/tests/data/era5-levels-members.grib",
        engine="gribberish"
    )

    # Get longitude coordinate
    lons = ds.longitude.values

    # Verify longitude dimensions (120 points at 3° resolution)
    assert len(lons) == 120, f"Expected 120 longitude points, got {len(lons)}"

    # Verify longitude range (0 to 357 at 3° steps)
    assert np.abs(lons[0] - 0.0) < 0.001, f"First longitude should be 0°, got {lons[0]}"
    assert np.abs(lons[-1] - 357.0) < 0.001, f"Last longitude should be 357°, got {lons[-1]}"

    # All longitudes should be in valid range [0, 360)
    assert np.all(lons >= 0.0), "All longitudes should be >= 0"
    assert np.all(lons < 360.0), "All longitudes should be < 360"


def test_latitude_values():
    """Test that latitude values are correct for global grids."""
    import numpy as np

    ds = xr.open_dataset(
        "./../gribberish/tests/data/geavg.t12z.pgrb2a.0p50.f000",
        engine="gribberish"
    )

    # Get latitude coordinate
    lats = ds.latitude.values

    # Verify latitude dimensions (361 points at 0.5° resolution from 90 to -90)
    assert len(lats) == 361, f"Expected 361 latitude points, got {len(lats)}"

    # Verify latitude range (90 to -90)
    assert np.abs(lats[0] - 90.0) < 0.001, f"First latitude should be 90°, got {lats[0]}"
    assert np.abs(lats[-1] - (-90.0)) < 0.001, f"Last latitude should be -90°, got {lats[-1]}"

    # All latitudes should be in valid range [-90, 90]
    assert np.all(lats >= -90.0), "All latitudes should be >= -90"
    assert np.all(lats <= 90.0), "All latitudes should be <= 90"
