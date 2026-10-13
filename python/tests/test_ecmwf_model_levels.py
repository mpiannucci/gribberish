"""ECMWF model-level fields: `cat` (0/19/29), `lnsp` (0/3/25) and the `pv` coordinate array."""

from pathlib import Path

import gribberish
import numpy as np
import pytest

FIXTURE = Path(__file__).parent.parent.parent / "test-data" / "ecmwf_ml_cat_lnsp_tiny.grib2"
PV_LEN = 184
PV_HEAD = [0.0, 2.000040054321289, 3.980832099914551]


def test_metadata_exposes_pv():
    raw = FIXTURE.read_bytes()
    meta = gribberish.parse_grib_message_metadata(raw, 0)
    assert meta.var_abbrev == "CAT"
    assert meta.level_type == "hybrid level"
    pv = meta.vertical_coordinate_values
    assert len(pv) == PV_LEN
    np.testing.assert_allclose(pv[:3], PV_HEAD, rtol=1e-6)
    assert pv[-1] == 1.0


def test_xarray_backend_reads_model_levels_with_pv():
    xr = pytest.importorskip("xarray")
    ds = xr.open_dataset(FIXTURE, engine="gribberish", collapse_groups=True)
    assert set(ds.data_vars) == {"cat", "lnsp"}
    assert ds["cat"].attrs["unit"] == "m2/3s-1"
    for name in ("cat", "lnsp"):
        pv = np.asarray(ds[name].attrs["pv"])
        assert pv.shape == (PV_LEN,)
        np.testing.assert_allclose(pv[:3], PV_HEAD, rtol=1e-6)
    np.testing.assert_allclose(float(ds["lnsp"].values.ravel()[0]), 11.52608871459961, atol=1e-5)
