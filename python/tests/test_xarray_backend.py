import numpy as np
import xarray as xr


def test_default_dtypes():
    ds = xr.open_dataset("./../gribberish/tests/data/hrrr.t06z.wrfsfcf01-TMP.grib2", engine="gribberish")
    # Data variables should be float32 by default
    for var_name, var in ds.data_vars.items():
        assert var.dtype == np.float32, f"Data variable {var_name} should be float32, got {var.dtype}"
    # Coordinates should always be float64
    for coord_name in ["latitude", "longitude", "x", "y"]:
        if coord_name in ds.coords:
            coord = ds.coords[coord_name]
            assert coord.dtype == np.float64, f"Coordinate {coord_name} should be float64, got {coord.dtype}"


def test_cfgrib_compat_removes_xy_coordinates():
    ds = xr.open_dataset("./../gribberish/tests/data/hrrr.t06z.wrfsfcf01-TMP.grib2", engine="gribberish", cfgrib_compat=True)
    assert "x" not in ds.coords, "x coordinate should be removed with cfgrib_compat=True"
    assert "y" not in ds.coords, "y coordinate should be removed with cfgrib_compat=True"
    assert "x" in ds.dims, "x dimension should still exist with cfgrib_compat=True"
    assert "y" in ds.dims, "y dimension should still exist with cfgrib_compat=True"


def test_float64_values_dtype():
    ds = xr.open_dataset(
        "./../gribberish/tests/data/hrrr.t06z.wrfsfcf01-TMP.grib2",
        engine="gribberish",
        backend_kwargs={"values_dtype": np.dtype("float64")}
    )
    for var_name, var in ds.data_vars.items():
        assert var.dtype == np.float64, f"Data variable {var_name} should be float64 when values_dtype=float64, got {var.dtype}"
    # Coordinates should still be float64
    for coord_name in ["latitude", "longitude", "x", "y"]:
        if coord_name in ds.coords:
            coord = ds.coords[coord_name]
            assert coord.dtype == np.float64, f"Coordinate {coord_name} should be float64, got {coord.dtype}"
