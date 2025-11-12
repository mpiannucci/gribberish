import pytest

xr = pytest.importorskip("xarray")


def test_xarray_backend_gefs_ensemble():
    """Test reading GEFS ensemble average file using xarray backend"""
    # Open the GEFS ensemble average file using the gribberish backend
    ds = xr.open_dataset(
        "./../gribberish/tests/data/geavg.t12z.pgrb2a.0p50.f000",
        engine="gribberish"
    )

    # Verify dataset was loaded
    assert ds is not None
    assert len(ds.data_vars) > 0

    # Check that we can access data variables
    for var_name in ds.data_vars:
        var = ds[var_name]
        assert var is not None
        # Verify we can load the data (tests lazy loading)
        data = var.values
        assert data is not None
        assert data.size > 0

    # Verify no duplicate keys (each variable should be unique)
    var_names = list(ds.data_vars.keys())
    assert len(var_names) == len(set(var_names)), "Found duplicate variable names"

    print(f"Successfully loaded {len(ds.data_vars)} variables from GEFS ensemble file")
    print(f"Variables: {list(ds.data_vars.keys())[:5]}...")  # Print first 5 variables
