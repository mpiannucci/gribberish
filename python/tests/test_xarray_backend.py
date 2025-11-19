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

    # Ensure all expected variables are present
    assert list(ds.variables) == ['weasd',
                                  'vgrd_VGRDhag_ens',
                                  'hgt_HGTisobar_ens',
                                  'rh_RHhag_ens',
                                  'rh_RHisobar_ens',
                                    'pwat',
                                    'icetk',
                                    'tmp_TMPisobar_ens',
                                    'vgrd_VGRDisobar_ens',
                                    'cin',
                                    'ugrd_UGRDisobar_ens',
                                    'tmp_TMPhag_ens',
                                    'ugrd_UGRDhag_ens',
                                    'prmsl',
                                    'cape',
                                    'tsoil',
                                    'pres',
                                    'snod',
                                    'hgt_HGTsfc_ens',
                                    'time',
                                    'isobar_0',
                                    'isobar_1',
                                    'isobar_2',
                                    'latitude',
                                    'longitude']
    
    # Check dimensions of 2D variables
    assert ds.cape.values.shape == (1, 361, 720)

    # Check dimension of 3D variables
    assert ds.tmp_TMPisobar_ens.values.shape == (1, 10, 361, 720)


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
