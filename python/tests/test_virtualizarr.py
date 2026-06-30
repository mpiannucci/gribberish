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


def _collapsed_store_for(filename, **kwargs):
    """A store built with the legacy collapse-to-root layout, for tests that
    just need a flat single dataset to exercise an orthogonal feature."""
    return _store_for(filename, collapse_groups=True, **kwargs)


def test_stable_layout_nests_under_level_and_kind():
    """By default every variable is nested under its surface-type and product
    kind, even when nothing in the file conflicts — so the group path is a pure
    function of the variable's own metadata. This ERA5 EDA file has a single
    isobaric instantaneous hypercube (10 GRIB1 members), which lands at
    `/isobar/instant` rather than collapsing to the root."""
    store = _store_for("era5-levels-members.grib")
    vdt = store.to_virtual_datatree()

    assert set(vdt.children) == {"isobar"}
    iso = vdt["isobar/instant"].to_dataset()
    assert {"t", "z"}.issubset(set(iso.data_vars))
    assert dict(iso.sizes) == {
        "time": 4,
        "isobar": 2,
        "number": 10,
        "latitude": 61,
        "longitude": 120,
    }
    assert iso["t"].dims == ("time", "isobar", "number", "latitude", "longitude")


def test_collapse_groups_single_root_dataset():
    """`collapse_groups=True` folds a conflict-free file back into a single
    virtual dataset, with levels and ensemble members as dimensions."""
    store = _collapsed_store_for("era5-levels-members.grib")
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


def test_group_paths_are_content_independent():
    """The whole point of the stable default: the same variable lands at the
    same group path regardless of what else is in the file.

    `tmp` exists at the surface in both fixtures, but only spans a second level
    (height-above-ground) in the NBM file. Under the legacy `collapse_groups`
    layout the surface `tmp` lands at `/` in the single-level file but `/sfc` in
    the multi-level one — different paths for the same variable, which breaks
    concatenation across a forecast sequence. The default layout pins it to
    `/sfc/instant` in both."""
    single_level = "hrrr.t06z.wrfsfcf01-TMP.grib2"  # tmp at sfc only
    multi_level = "nbm-multilevel-tcdc.grib2"        # tmp at hag AND sfc

    def tmp_paths(filename, **kwargs):
        vdt = _store_for(filename, **kwargs).to_virtual_datatree()
        return sorted(
            node.path
            for node in vdt.subtree
            if "tmp" in vdt[node.path].dataset.data_vars
        )

    # Stable default: surface tmp is at /sfc/instant in both files.
    assert "/sfc/instant" in tmp_paths(single_level)
    assert "/sfc/instant" in tmp_paths(multi_level)

    # Legacy collapse layout: the surface tmp's path depends on the file.
    assert tmp_paths(single_level, collapse_groups=True) == ["/"]
    assert "/sfc" in tmp_paths(multi_level, collapse_groups=True)
    assert "/sfc/instant" not in tmp_paths(multi_level, collapse_groups=True)


def test_conflict_produces_datatree_groups():
    """Conflicting product kinds become a DataTree of standalone groups. These
    S2S products are all at height-above-ground, so they nest under `/hag`."""
    store = _store_for("s2s-pdt9-pdt10-pdt12.grib2")
    vdt = store.to_virtual_datatree()

    assert set(vdt.children) == {"hag"}
    groups = set(vdt["hag"].children)
    # percentile product is its own group, with the percentile dim inside it
    assert "min24h_pctl" in groups
    pctl = store.to_virtual_datatree()["hag/min24h_pctl"]
    assert "percentile" in pctl.dims
    # between-limit probabilities split per (lower, upper) pair
    assert len([g for g in groups if "prob_between_inc" in g]) == 3


def test_level_type_conflict_groups():
    """A variable spanning multiple level types splits by level type; each level
    then nests by product kind (this GEFS file is an ensemble mean)."""
    store = _store_for("geavg.t12z.pgrb2a.0p50.f000")
    vdt = store.to_virtual_datatree()
    assert {"hag", "isobar", "sfc", "msl"}.issubset(set(vdt.children))

    iso = vdt["isobar/mean"].to_dataset()
    assert {"tmp", "ugrd", "vgrd", "rh", "hgt"}.issubset(set(iso.data_vars))
    # isobaric temperature keeps its vertical level dimension
    assert iso["tmp"].shape == (1, 10, 361, 720)


def test_missing_level_does_not_break_datatree():
    """A variable with a missing/unrecognized surface type must not create an
    unnamed group. It keeps its product-kind segment but no level segment, so it
    lands at a top-level kind group (`/instant`) rather than an empty-named one.
    The fixture has TMP at two levels plus TCDC on a "reserved" surface that
    gribberish maps to a missing level type.
    """
    store = _store_for("nbm-multilevel-tcdc.grib2")
    vdt = store.to_virtual_datatree()

    keys = {k for k, _ in vdt.subtree_with_keys}
    assert "" not in keys, "an unnamed group leaked into the hierarchy"
    # The well-formed variable still splits by level type.
    assert {"hag", "sfc"}.issubset(set(vdt.children))
    # The missing-level variable is preserved in a top-level kind group.
    assert "tcdc" in vdt["instant"].dataset.data_vars


def test_decoded_values_match_direct_parse():
    """Reading a chunk through the store decodes the exact message the manifest
    points to (the gribberish codec resolves at read time)."""
    fname = "era5-levels-members.grib"
    store = _collapsed_store_for(fname)
    z = zarr.open(store, mode="r")

    chunk = np.asarray(z["t"][0, 0, 0])  # (time0, isobar0, number0) -> (lat, lon)
    assert chunk.shape == (61, 120)

    with open(TEST_DATA / fname, "rb") as f:
        data = f.read()
    ds = parse_grib_dataset(data, collapse_groups=True)
    offset, size = ds["data_vars"]["t"]["values"]["offsets"][0]
    expected = parse_grib_array(data[offset : offset + size], 0).reshape(61, 120)

    np.testing.assert_allclose(chunk, expected)


def test_inline_coordinates_decode():
    """Derived coordinates (time / level) are inlined and decode correctly."""
    fname = "era5-levels-members.grib"
    store = _collapsed_store_for(fname)
    z = zarr.open(store, mode="r")

    isobar = np.asarray(z["isobar"][:])
    np.testing.assert_array_equal(isobar, [500.0, 850.0])
    # time is CF-encoded int64 seconds with a units attribute
    assert z["time"].metadata.data_type.to_native_dtype() == np.dtype("int64")
    assert "since" in z["time"].attrs["units"]


def test_projected_grid_latlon_are_references():
    """A projected (Lambert) grid keeps 2D lat/lon as byte references decoded
    by the gribberish codec."""
    store = _collapsed_store_for("hrrr.t06z.wrfsfcf01-TMP.grib2")
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
        store = _collapsed_store_for(fname)
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
    store = _collapsed_store_for("aifs-ens-cf-t500.grib2")
    vds = store.to_virtual_dataset()

    assert "number" in vds.coords
    assert vds["tmp"].dims == ("time", "number", "latitude", "longitude")


def test_drop_and_only_variables():
    store = _collapsed_store_for("ecmwf-ifs-oper-surface.grib2", only_variables=["tcc"])
    vds = store.to_virtual_dataset()
    assert set(vds.data_vars) == {"tcc"}


def test_use_index_matches_full_scan():
    """Building through the sidecar .idx — header-only range reads, no data
    sections fetched — produces the same virtual store as scanning the whole
    file: identical structure, identical chunk manifests (real file offsets),
    identical decoded values."""
    fname = "gfswave.t18z.atlocn.0p16.f001.grib2"
    full = _collapsed_store_for(fname).to_virtual_dataset()
    via_index = _collapsed_store_for(fname, use_index=".idx").to_virtual_dataset()

    assert dict(via_index.sizes) == dict(full.sizes)
    assert set(via_index.data_vars) == set(full.data_vars)
    for name in full.data_vars:
        assert via_index[name].data.manifest == full[name].data.manifest

    z_full = zarr.open(_collapsed_store_for(fname), mode="r")
    z_index = zarr.open(_collapsed_store_for(fname, use_index=".idx"), mode="r")
    np.testing.assert_array_equal(
        np.asarray(z_index["wind"][0]), np.asarray(z_full["wind"][0])
    )


def test_adjust_longitude_range_global_grid():
    """Opt-in longitude wrap: a 0–360° GFS grid becomes a monotonic −180…180°
    store whose data is rolled to match, so a box straddling the prime meridian
    slices cleanly (the gotcha from issue #156)."""
    fname = "gfs.t18z.pgrb2.0p25.f186-RH.grib2"
    nx, roll = 1440, 720

    plain = _collapsed_store_for(fname).to_virtual_dataset()
    wrapped = _collapsed_store_for(
        fname, adjust_longitude_range=True
    ).to_virtual_dataset()

    # default coordinate is the native 0..360 range
    np.testing.assert_array_equal(plain.longitude.values[[0, -1]], [0.0, 359.75])

    # wrapped coordinate is strictly monotonic over [-180, 180)
    lon = wrapped.longitude.values
    assert lon[0] == -180.0 and lon[-1] == 179.75
    assert np.all(np.diff(lon) > 0)

    # data stays aligned with the coordinate: reading through the wrapped store
    # equals the plain store rolled along longitude by the same split column.
    z_plain = zarr.open(_collapsed_store_for(fname), mode="r")
    z_wrapped = zarr.open(
        _collapsed_store_for(fname, adjust_longitude_range=True), mode="r"
    )
    plain_data = np.asarray(z_plain["rh"][...])
    wrapped_data = np.asarray(z_wrapped["rh"][...])
    cols = (np.arange(nx) + roll) % nx
    np.testing.assert_array_equal(wrapped_data, plain_data[..., cols])

    # the issue's acceptance: with the wrapped coordinate + rolled data assembled
    # into a dataset (as a consumer like icechunk does at read time), a box around
    # the prime meridian slices cleanly to 161 columns — the native 0–360 store
    # would silently drop the [-10, 0) half.
    import xarray as xr

    ds = xr.Dataset(
        {"rh": (("time", "latitude", "longitude"), wrapped_data)},
        coords={"longitude": lon},
    )
    box = ds.sel(longitude=slice(-10, 30))
    assert box.sizes["longitude"] == 161
    np.testing.assert_array_equal(box.longitude.values[[0, -1]], [-10.0, 30.0])


def test_adjust_longitude_range_start_at_antimeridian():
    """A grid that already starts at 180° (ECMWF/AIFS) is non-monotonic in 0–360
    today; wrapping relabels the coordinate to monotonic −180…180 with no data
    move (roll == 0), exercising the relabel-only branch."""
    fname = "aifs-single-t500.grib2"

    plain = _collapsed_store_for(fname).to_virtual_dataset()
    wrapped = _collapsed_store_for(
        fname, adjust_longitude_range=True
    ).to_virtual_dataset()

    # native coordinate wraps mid-array (180..359.75, then 0..179.75)
    assert plain.longitude.values[0] == 180.0 and plain.longitude.values[-1] == 179.75
    assert not np.all(np.diff(plain.longitude.values) > 0)

    # wrapped coordinate is monotonic -180..180
    lon = wrapped.longitude.values
    assert lon[0] == -180.0 and lon[-1] == 179.75
    assert np.all(np.diff(lon) > 0)

    # data is unchanged because the columns were already in -180..180 order
    z_plain = zarr.open(_collapsed_store_for(fname), mode="r")
    z_wrapped = zarr.open(
        _collapsed_store_for(fname, adjust_longitude_range=True), mode="r"
    )
    np.testing.assert_array_equal(
        np.asarray(z_wrapped["tmp"][...]), np.asarray(z_plain["tmp"][...])
    )


def test_adjust_longitude_range_default_off_unchanged():
    """Default-off parser is byte-for-byte the existing behaviour."""
    fname = "gfs.t18z.pgrb2.0p25.f186-RH.grib2"
    default = _collapsed_store_for(fname).to_virtual_dataset()
    explicit_off = _collapsed_store_for(
        fname, adjust_longitude_range=False
    ).to_virtual_dataset()
    np.testing.assert_array_equal(
        default.longitude.values, explicit_off.longitude.values
    )
    assert default.longitude.values[0] == 0.0


def test_adjust_longitude_values_helper():
    """The longitude-wrap kernel the parser calls on the inlined coordinate:
    a global 0..360 axis becomes monotonic -180..180; a non-global axis is
    returned unchanged."""
    from gribberish import adjust_longitude_values

    # Global 0.25° axis (1440 pts, 0..359.75): split at 180° -> -180..179.75.
    native = np.arange(1440) * 0.25
    wrapped = np.asarray(adjust_longitude_values(native))
    assert wrapped[0] == -180.0 and wrapped[-1] == 179.75
    assert np.all(np.diff(wrapped) > 0)

    # Regional axis (90° wide) is not near-global -> returned unchanged.
    regional = 10.0 + np.arange(360) * 0.25
    np.testing.assert_array_equal(
        np.asarray(adjust_longitude_values(regional)), regional
    )


def test_layer_variable_distinguished_by_second_surface():
    """Layer quantities whose bottom surface is constant but top varies (HRRR
    0-1000m vs 0-6000m wind shear) must not collapse into a single chunk slot.
    The second (top) fixed surface becomes the vertical coordinate so each
    layer maps to its own message."""
    store = _collapsed_store_for("hrrr.t01z.wrfsfcf01-VVCSH-VUCSH.grib2")
    vds = store.to_virtual_dataset()

    assert {"vvcsh", "vucsh"}.issubset(set(vds.data_vars))
    # one vertical dimension of length 2, the layer tops 1000 m and 6000 m
    (vert_dim,) = [d for d in vds["vvcsh"].dims if d not in ("time", "y", "x")]
    assert vds.sizes[vert_dim] == 2
    np.testing.assert_array_equal(
        np.asarray(vds.coords[vert_dim].values), [1000.0, 6000.0]
    )
    assert vds["vvcsh"].shape == (1, 2, 1059, 1799)
