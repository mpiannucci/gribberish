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
