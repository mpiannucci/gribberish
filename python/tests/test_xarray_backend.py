import numpy as np
import xarray as xr


def test_cfgrib_compat_removes_xy_coordinates():
    ds = xr.open_dataset("./../gribberish/tests/data/hrrr.t06z.wrfsfcf01-TMP.grib2", engine="gribberish", cfgrib_compat=True)
    assert "x" not in ds.coords, "x coordinate should be removed with cfgrib_compat=True"
    assert "y" not in ds.coords, "y coordinate should be removed with cfgrib_compat=True"
    assert "x" in ds.dims, "x dimension should still exist with cfgrib_compat=True"
    assert "y" in ds.dims, "y dimension should still exist with cfgrib_compat=True"
