from pathlib import Path

import pytest

pytest.importorskip("virtualizarr")
pytest.importorskip("obstore")
pytest.importorskip("zarr")

import numpy as np
import zarr
from obspec_utils.registry import ObjectStoreRegistry
from obstore.store import LocalStore

from gribberish import parse_grib_array, parse_grib_dataset
from gribberish.virtualizarr import GribberishParser

REPO_ROOT = Path(__file__).resolve().parents[2]
TEST_DATA = REPO_ROOT / "test-data"


def _store_for(filename, **kwargs):
    registry = ObjectStoreRegistry({"file://": LocalStore()})
    url = (TEST_DATA / filename).as_uri()
    return GribberishParser(**kwargs)(url, registry)


def test_no_conflict_single_root_dataset():
    """A conflict-free file becomes a single virtual dataset, levels and ensemble
    members as dimensions (this ERA5 EDA file has 10 GRIB1 members)."""
    store = _store_for("era5-levels-members.grib")
    vds = store.to_virtual_dataset()

    assert dict(vds.sizes) == {
        "time": 4,
        "isobar": 2,
        "number": 10,
        "latitude": 61,
        "longitude": 120,
    }
    assert {"time", "isobar", "number", "latitude", "longitude"}.issubset(set(vds.coords))
    assert {"t", "z"}.issubset(set(vds.data_vars))
    assert vds["t"].dims == ("time", "isobar", "number", "latitude", "longitude")


def test_conflict_produces_datatree_groups():
    """Conflicting product kinds become a DataTree of standalone groups."""
    store = _store_for("s2s-pdt9-pdt10-pdt12.grib2")
    vdt = store.to_virtual_datatree()

    groups = set(vdt.children)
    # percentile product is its own group, with the percentile dim inside it
    assert "min24h_pctl" in groups
    pctl = store.to_virtual_datatree()["min24h_pctl"]
    assert "percentile" in pctl.dims
    # between-limit probabilities split per (lower, upper) pair
    assert len([g for g in groups if "prob_between_inc" in g]) == 3


def test_level_type_conflict_groups():
    """A variable spanning multiple level types splits by level type."""
    store = _store_for("geavg.t12z.pgrb2a.0p50.f000")
    vdt = store.to_virtual_datatree()
    assert {"hag", "isobar", "sfc", "msl"}.issubset(set(vdt.children))

    iso = vdt["isobar"].to_dataset()
    assert {"tmp", "ugrd", "vgrd", "rh", "hgt"}.issubset(set(iso.data_vars))
    # isobaric temperature keeps its vertical level dimension
    assert iso["tmp"].shape == (1, 10, 361, 720)


def test_decoded_values_match_direct_parse():
    """Reading a chunk through the store decodes the exact message the manifest
    points to (the gribberish codec resolves at read time)."""
    fname = "era5-levels-members.grib"
    store = _store_for(fname)
    z = zarr.open(store, mode="r")

    chunk = np.asarray(z["t"][0, 0, 0])  # (time0, isobar0, number0) -> (lat, lon)
    assert chunk.shape == (61, 120)

    with open(TEST_DATA / fname, "rb") as f:
        data = f.read()
    ds = parse_grib_dataset(data)
    offset, size = ds["data_vars"]["t"]["values"]["offsets"][0]
    expected = parse_grib_array(data[offset : offset + size], 0).reshape(61, 120)

    np.testing.assert_allclose(chunk, expected)


def test_inline_coordinates_decode():
    """Derived coordinates (time / level) are inlined and decode correctly."""
    fname = "era5-levels-members.grib"
    store = _store_for(fname)
    z = zarr.open(store, mode="r")

    isobar = np.asarray(z["isobar"][:])
    np.testing.assert_array_equal(isobar, [500.0, 850.0])
    # time is CF-encoded int64 seconds with a units attribute
    assert z["time"].metadata.data_type.to_native_dtype() == np.dtype("int64")
    assert "since" in z["time"].attrs["units"]


def test_projected_grid_latlon_are_references():
    """A projected (Lambert) grid keeps 2D lat/lon as byte references decoded
    by the gribberish codec."""
    store = _store_for("hrrr.t06z.wrfsfcf01-TMP.grib2")
    vds = store.to_virtual_dataset()

    assert vds["latitude"].dims == ("y", "x")
    assert vds["longitude"].dims == ("y", "x")

    z = zarr.open(store, mode="r")
    lat = np.asarray(z["latitude"][:])
    assert lat.shape == (1059, 1799)
    assert np.isfinite(lat).all()


def test_cf_grid_mapping_scalar_coordinate():
    """A scalar `spatial_ref` coordinate carries the CF grid mapping, and every
    data variable links to it via a `grid_mapping` attribute. The 0-d coordinate
    must round-trip through the manifest without being padded to shape (1,)."""
    pyproj = pytest.importorskip("pyproj")

    # Lambert conformal (HRRR) and a plain lat/lon grid (GFS).
    cases = {
        "hrrr.t06z.wrfsfcf01-TMP.grib2": "lambert_conformal_conic",
        "gfs.t18z.pgrb2.0p25.f186-RH.grib2": "latitude_longitude",
    }
    for fname, grid_mapping_name in cases.items():
        store = _store_for(fname)
        vds = store.to_virtual_dataset()

        assert "spatial_ref" in vds.coords
        assert vds["spatial_ref"].shape == ()
        assert vds["spatial_ref"].attrs["grid_mapping_name"] == grid_mapping_name

        data_vars = [v for v in vds.data_vars if v != "spatial_ref"]
        assert data_vars
        for name in data_vars:
            assert vds[name].attrs["grid_mapping"] == "spatial_ref"

        # CF attributes reconstruct the same CRS as the proj4 string.
        attrs = vds["spatial_ref"].attrs
        cf = {k: v for k, v in attrs.items() if k != "proj4"}
        assert pyproj.CRS.from_cf(cf) == pyproj.CRS.from_proj4(attrs["proj4"])


def test_ensemble_member_dimension():
    """Ensemble member number is a dimension, not a group."""
    store = _store_for("aifs-ens-cf-t500.grib2")
    vds = store.to_virtual_dataset()

    assert "number" in vds.coords
    assert vds["tmp"].dims == ("time", "number", "latitude", "longitude")


def test_drop_and_only_variables():
    store = _store_for("ecmwf-ifs-oper-surface.grib2", only_variables=["tcc"])
    vds = store.to_virtual_dataset()
    assert set(vds.data_vars) == {"tcc"}


def test_use_index_matches_full_scan():
    """Building through the sidecar .idx — header-only range reads, no data
    sections fetched — produces the same virtual store as scanning the whole
    file: identical structure, identical chunk manifests (real file offsets),
    identical decoded values."""
    fname = "gfswave.t18z.atlocn.0p16.f001.grib2"
    full = _store_for(fname).to_virtual_dataset()
    via_index = _store_for(fname, use_index=".idx").to_virtual_dataset()

    assert dict(via_index.sizes) == dict(full.sizes)
    assert set(via_index.data_vars) == set(full.data_vars)
    for name in full.data_vars:
        assert via_index[name].data.manifest == full[name].data.manifest

    z_full = zarr.open(_store_for(fname), mode="r")
    z_index = zarr.open(_store_for(fname, use_index=".idx"), mode="r")
    np.testing.assert_array_equal(
        np.asarray(z_index["wind"][0]), np.asarray(z_full["wind"][0])
    )


def test_layer_variable_distinguished_by_second_surface():
    """Layer quantities whose bottom surface is constant but top varies (HRRR
    0-1000m vs 0-6000m wind shear) must not collapse into a single chunk slot.
    The second (top) fixed surface becomes the vertical coordinate so each
    layer maps to its own message."""
    store = _store_for("hrrr.t01z.wrfsfcf01-VVCSH-VUCSH.grib2")
    vds = store.to_virtual_dataset()

    assert {"vvcsh", "vucsh"}.issubset(set(vds.data_vars))
    # one vertical dimension of length 2, the layer tops 1000 m and 6000 m
    (vert_dim,) = [d for d in vds["vvcsh"].dims if d not in ("time", "y", "x")]
    assert vds.sizes[vert_dim] == 2
    np.testing.assert_array_equal(
        np.asarray(vds.coords[vert_dim].values), [1000.0, 6000.0]
    )
    assert vds["vvcsh"].shape == (1, 2, 1059, 1799)
