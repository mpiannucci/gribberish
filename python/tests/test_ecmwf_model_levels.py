"""ECMWF IFS model-level fields: `cat` (0/19/29), `lnsp` (0/3/25) and the `pv` coordinate array.

The fixture is a 20 x 25 sub-area cut with eccodes from two real messages of the IFS
model-level feed (12Z 2026-10-12, step 1); values and `pv` are the originals.
"""

from pathlib import Path

import gribberish
import numpy as np
import pytest

FIXTURE = Path(__file__).parent.parent.parent / "test-data" / "ecmwf_ifs_ml_cat_lnsp_subarea.grib2"
PV_LEN = 276
PV_HEAD = [0.0, 2.0003650188446045, 3.102241039276123]


def test_metadata_exposes_pv():
    raw = FIXTURE.read_bytes()
    meta = gribberish.parse_grib_message_metadata(raw, 0)
    assert meta.var_abbrev == "LNSP"
    assert meta.level_type == "hybrid level"
    pv = np.asarray(meta.vertical_coordinate_values)
    assert pv.shape == (PV_LEN,)
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
    np.testing.assert_allclose(float(ds["lnsp"].values.ravel()[0]), 11.432563781738281, atol=1e-9)
    np.testing.assert_allclose(float(ds["cat"].values.ravel()[0]), 0.01685124426148832, atol=1e-9)
