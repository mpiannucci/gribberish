import pytest
from pathlib import Path

xr = pytest.importorskip("xarray")

from gribberish.gribberish_backend import GribberishBackend


def test_xarray_backend_gefs_ensemble():
    """GEFS ensemble-mean file splits into standalone groups by level type."""
    path = "./../test-data/geavg.t12z.pgrb2a.0p50.f000"

    # Opening without a group raises, listing the available groups to choose.
    with pytest.raises(ValueError, match="multiple groups"):
        xr.open_dataset(path, engine="gribberish")

    # The whole file is available as a DataTree of standalone group datasets.
    dt = xr.open_datatree(path, engine="gribberish")
    assert {"hag", "isobar", "sfc", "msl"}.issubset(set(dt.children))

    # Open one group directly; variables keep their plain short names.
    iso = xr.open_dataset(path, engine="gribberish", group="isobar")
    assert {"tmp", "ugrd", "vgrd", "rh", "hgt"}.issubset(set(iso.data_vars))
    assert "time" in iso.coords
    assert "latitude" in iso.coords
    assert "longitude" in iso.coords
    # tmp at isobaric levels: (time, isobar_*, lat, lon)
    assert iso.tmp.values.shape == (1, 10, 361, 720)

    # Single-level fields live in their level-type group with no vertical dim.
    hag = xr.open_dataset(path, engine="gribberish", group="hag")
    assert hag.tmp.values.shape == (1, 361, 720)
    cape = xr.open_dataset(path, engine="gribberish", group="pres_diff")
    assert cape.cape.values.shape == (1, 361, 720)


def test_xarray_filters_collapse_single_remaining_group():
    """Variable/attribute filters can resolve conflicting hypercubes."""
    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "test-data" / "geavg.t12z.pgrb2a.0p50.f000"

    ds = xr.open_dataset(str(fixture), engine="gribberish", only_variables=["prmsl"])
    assert set(ds.data_vars) == {"prmsl"}
    assert tuple(ds.prmsl.shape) == (1, 361, 720)

    ds = xr.open_dataset(
        str(fixture),
        engine="gribberish",
        filter_by_attrs={"fixed_surface_type": "mean sea level"},
    )
    assert set(ds.data_vars) == {"prmsl"}
    assert tuple(ds.prmsl.shape) == (1, 361, 720)


def test_xarray_backend_era5_grib1():
    """Test reading ERA5 GRIB1 file with levels and members using xarray backend"""
    # Open the ERA5 GRIB1 file using the gribberish backend
    ds = xr.open_dataset("./../test-data/era5-levels-members.grib", engine="gribberish")

    # Verify dataset was loaded
    assert ds is not None

    # Check that we have the expected variable (geopotential)
    assert "z" in ds.variables or "geopotential" in ds.data_vars.keys()

    # Check coordinates are present
    assert "latitude" in ds.variables
    assert "longitude" in ds.variables
    assert "time" in ds.variables

    # Verify dimensions based on the Rust test expectations (61, 120)
    # The data array should have shape (time, lat, lon) or similar
    geopotential_var = "z" if "z" in ds.data_vars else list(ds.data_vars.keys())[0]
    data_shape = ds[geopotential_var].values.shape

    # Check that lat/lon dimensions match expected (61, 120)
    # Note: Order might be (time, lat, lon) so we check the last two dimensions
    assert data_shape[-2:] == (61, 120) or data_shape[-2:] == (120, 61), (
        f"Expected grid dimensions (61, 120) but got {data_shape}"
    )

    # Verify the data can be loaded
    data = ds[geopotential_var].values
    assert data.size > 0

    # Check that the data is not all zeros or NaNs
    import numpy as np

    assert not np.all(data == 0), "Data should not be all zeros"
    assert not np.all(np.isnan(data)), "Data should not be all NaNs"


def test_xarray_backend_era5_grib1_ensemble():
    """ERA5 GRIB1 EDA file exposes its 10 ensemble members as a 'number' dim."""
    import numpy as np

    ds = xr.open_dataset(
        "./../test-data/era5-levels-members.grib", engine="gribberish"
    )

    # 10 ensemble members decoded from the ECMWF GRIB1 PDS local extension
    assert "number" in ds.coords
    assert ds.sizes["number"] == 10
    np.testing.assert_array_equal(ds.number.values, np.arange(10))
    # No conflict -> members are a dimension, not a group
    assert "number" in ds.t.dims


def test_xarray_backend_can_open_grib1_extensions():
    backend = GribberishBackend()
    assert backend.guess_can_open("example.grib1")
    assert backend.guess_can_open("example.GRIB1")
    assert backend.guess_can_open("example.grib")
    assert backend.guess_can_open("example.grib2")


def test_xarray_open_datatree_can_guess_gribberish_engine():
    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "test-data" / "hrrr.t06z.wrfsfcf01-TMP.grib2"

    dt = xr.open_datatree(str(fixture))
    assert set(dt.to_dataset().data_vars) == {"tmp"}


def test_xarray_backend_complex_packing_missing_values():
    """Complex packing with 2nd-order spatial differencing AND primary missing
    values (GFS temperature on the potential-vorticity surface). Regression test:
    the all-ones missing groups used to be decoded as real data, blowing the
    second-order reconstruction up to ~2e8 across 99% of the grid. Values and
    NaN mask validated against cfgrib/eccodes."""
    import numpy as np

    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "test-data" / "gfs.t12z.pgrb2.0p25.f023-PV-TMP-missing.grib2"
    ds = xr.open_dataset(str(fixture), engine="gribberish")

    arr = np.asarray(ds["tmp"].values)
    finite = np.isfinite(arr)
    assert int(finite.sum()) == 629160
    assert int((~finite).sum()) == 409080
    assert 184.0 < float(np.nanmin(arr)) < 186.0
    assert 289.0 < float(np.nanmax(arr)) < 290.0


def test_xarray_backend_ecmwf_soil_tiny_fixture():
    """Test reading ECMWF soil GRIB1 file with 8 soil variables."""
    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "test-data" / "ecmwf_soil_8vars_tiny.grib1"
    ds = xr.open_dataset(str(fixture), engine="gribberish")

    # Should have 8 soil variables: 4 soil temperature layers + 4 soil moisture layers
    expected_vars = {"stl1", "stl2", "stl3", "stl4", "swvl1", "swvl2", "swvl3", "swvl4"}
    assert set(ds.data_vars.keys()) == expected_vars, (
        f"Expected {expected_vars}, got {set(ds.data_vars.keys())}"
    )

    # All variables should have the same grid shape (4x4 tiny grid)
    for var in expected_vars:
        assert ds[var].values.shape == (1, 4, 4), (
            f"Expected shape (1, 4, 4) for {var}, got {ds[var].values.shape}"
        )


def test_multiple_variables_at_same_level():
    """Multiple variables at the same level type share a group, each a distinct
    variable with its own data."""
    import numpy as np

    ds = xr.open_dataset(
        "./../test-data/geavg.t12z.pgrb2a.0p50.f000", engine="gribberish", group="hag"
    )

    # Multiple variables exist in the 'hag' (height above ground) group
    for var in ["tmp", "ugrd", "vgrd", "rh"]:
        assert var in ds.data_vars, f"Expected variable {var} to exist"

    # Verify these are genuinely different variables with different data
    tmp_data = ds.tmp.values
    ugrd_data = ds.ugrd.values
    vgrd_data = ds.vgrd.values

    assert not np.allclose(tmp_data, ugrd_data, equal_nan=True), (
        "TMP and UGRD should have different data"
    )
    assert not np.allclose(tmp_data, vgrd_data, equal_nan=True), (
        "TMP and VGRD should have different data"
    )
    assert not np.allclose(ugrd_data, vgrd_data, equal_nan=True), (
        "UGRD and VGRD should have different data"
    )

    # Each variable should have proper attributes identifying its parameter
    assert (
        "tmp" in ds.tmp.attrs.get("standard_name", "").lower()
        or "temperature" in ds.tmp.attrs.get("standard_name", "").lower()
    )
    assert (
        "wind" in ds.ugrd.attrs.get("standard_name", "").lower()
        or "u-component" in ds.ugrd.attrs.get("standard_name", "").lower()
    )


def test_variable_naming_without_redundant_prefix():
    """Variables keep plain short names now that hypercube differences are
    expressed as groups rather than name suffixes."""
    dt = xr.open_datatree(
        "./../test-data/geavg.t12z.pgrb2a.0p50.f000", engine="gribberish"
    )

    for node in dt.subtree:
        for var_name in node.data_vars:
            assert var_name == var_name.lower(), (
                f"Variable name '{var_name}' should be lowercase"
            )
            # No level-type / product suffix is baked into the name anymore.
            for suffix in ("_ensmean", "_isobar", "_hag", "_sfc", "_fcst"):
                assert suffix not in var_name, (
                    f"Variable name '{var_name}' should not carry suffix '{suffix}'"
                )


def test_longitude_normalization_grib2():
    """Test that longitude values are correctly normalized for global GRIB2 grids.

    This validates the fix for longitude wrapping - all longitude values should
    be in a valid range [0, 360) for global grids.
    """
    import numpy as np

    ds = xr.open_dataset(
        "./../test-data/geavg.t12z.pgrb2a.0p50.f000", engine="gribberish", group="hag"
    )

    # Get longitude coordinate
    lons = ds.longitude.values

    # Verify longitude dimensions (720 points at 0.5° resolution)
    assert len(lons) == 720, f"Expected 720 longitude points, got {len(lons)}"

    # Verify longitude range (0 to 359.5 at 0.5° steps)
    assert np.abs(lons[0] - 0.0) < 0.001, f"First longitude should be 0°, got {lons[0]}"
    assert np.abs(lons[-1] - 359.5) < 0.001, (
        f"Last longitude should be 359.5°, got {lons[-1]}"
    )

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

    ds = xr.open_dataset("./../test-data/era5-levels-members.grib", engine="gribberish")

    # Get longitude coordinate
    lons = ds.longitude.values

    # Verify longitude dimensions (120 points at 3° resolution)
    assert len(lons) == 120, f"Expected 120 longitude points, got {len(lons)}"

    # Verify longitude range (0 to 357 at 3° steps)
    assert np.abs(lons[0] - 0.0) < 0.001, f"First longitude should be 0°, got {lons[0]}"
    assert np.abs(lons[-1] - 357.0) < 0.001, (
        f"Last longitude should be 357°, got {lons[-1]}"
    )

    # All longitudes should be in valid range [0, 360)
    assert np.all(lons >= 0.0), "All longitudes should be >= 0"
    assert np.all(lons < 360.0), "All longitudes should be < 360"


def test_latitude_values():
    """Test that latitude values are correct for global grids."""
    import numpy as np

    ds = xr.open_dataset(
        "./../test-data/geavg.t12z.pgrb2a.0p50.f000", engine="gribberish", group="hag"
    )

    # Get latitude coordinate
    lats = ds.latitude.values

    # Verify latitude dimensions (361 points at 0.5° resolution from 90 to -90)
    assert len(lats) == 361, f"Expected 361 latitude points, got {len(lats)}"

    # Verify latitude range (90 to -90)
    assert np.abs(lats[0] - 90.0) < 0.001, (
        f"First latitude should be 90°, got {lats[0]}"
    )
    assert np.abs(lats[-1] - (-90.0)) < 0.001, (
        f"Last latitude should be -90°, got {lats[-1]}"
    )

    # All latitudes should be in valid range [-90, 90]
    assert np.all(lats >= -90.0), "All latitudes should be >= -90"
    assert np.all(lats <= 90.0), "All latitudes should be <= 90"


def test_xarray_backend_aifs_ensemble():
    """Test reading AIFS ensemble GRIB2 file with ensemble member dimension."""
    repo_root = Path(__file__).resolve().parents[2]
    fixture = repo_root / "test-data" / "aifs-ens-cf-t500.grib2"
    ds = xr.open_dataset(str(fixture), engine="gribberish")

    assert "tmp" in ds.data_vars
    assert "number" in ds.coords
    assert ds.number.values.shape == (1,)

    assert ds.tmp.dims == ("time", "number", "latitude", "longitude")
    assert ds.tmp.values.shape == (1, 1, 721, 1440)


def test_xarray_backend_s2s_percentile_probability():
    """S2S GRIB2 files with PDT 9/10/12 split into groups by product kind.

    PDT 10 (percentile) -> its own group with a 'percentile' dimension.
    PDT 9 (probability) -> a group per probability type / limit pair.
    PDT 12 (derived ensemble) -> mean groups.
    """
    import numpy as np

    path = "./../test-data/s2s-pdt9-pdt10-pdt12.grib2"

    # Conflicting product kinds -> opening without a group lists them.
    with pytest.raises(ValueError, match="multiple groups"):
        xr.open_dataset(path, engine="gribberish")

    dt = xr.open_datatree(path, engine="gribberish")
    groups = set(dt.children)

    # The percentile product is its own group, with a percentile dimension.
    assert "min24h_pctl" in groups, f"groups: {sorted(groups)}"
    pctl = xr.open_dataset(path, engine="gribberish", group="min24h_pctl")
    assert "percentile" in pctl.coords
    np.testing.assert_array_equal(sorted(pctl.percentile.values), [1, 50, 99])
    assert "percentile" in pctl.tmp.dims

    # Between-limit probabilities split into a group per (lower, upper) pair.
    between = [g for g in groups if "prob_between_inc" in g]
    assert len(between) == 3, f"expected 3 between_inc groups, got {sorted(between)}"

    # Probability groups never carry a percentile dimension.
    for gname in (g for g in groups if "prob" in g):
        prob = xr.open_dataset(path, engine="gribberish", group=gname)
        assert "percentile" not in prob.tmp.dims


def test_cf_grid_mapping_metadata():
    """The backend emits a scalar `spatial_ref` grid-mapping coordinate and
    links each variable to it, so geospatial tooling can recover the CRS."""
    pyproj = pytest.importorskip("pyproj")

    cases = {
        "./../test-data/hrrr.t06z.wrfsfcf01-TMP.grib2": "lambert_conformal_conic",
        "./../test-data/gfs.t18z.pgrb2.0p25.f186-RH.grib2": "latitude_longitude",
    }
    for path, grid_mapping_name in cases.items():
        ds = xr.open_dataset(path, engine="gribberish")

        assert "spatial_ref" in ds.coords
        assert ds["spatial_ref"].shape == ()
        assert ds["spatial_ref"].attrs["grid_mapping_name"] == grid_mapping_name

        for name in ds.data_vars:
            assert ds[name].attrs["grid_mapping"] == "spatial_ref"

        attrs = ds["spatial_ref"].attrs
        cf = {k: v for k, v in attrs.items() if k != "proj4"}
        assert pyproj.CRS.from_cf(cf) == pyproj.CRS.from_proj4(attrs["proj4"])
