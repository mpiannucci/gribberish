from pathlib import Path

import pytest

pytest.importorskip("virtualizarr")
pytest.importorskip("obstore")
pytest.importorskip("zarr")

import numpy as np
import zarr
from obspec_utils.registry import ObjectStoreRegistry
from obstore.store import LocalStore

from gribberish.virtualizarr import GribberishParser

REPO_ROOT = Path(__file__).resolve().parents[2]
TEST_DATA = REPO_ROOT / "test-data"
# GEFS: isobaric temperature/rh at 10 levels, hgt at 11, ugrd/vgrd at 12.
FIXTURE = "geavg.t12z.pgrb2a.0p50.f000"


def _iso_store_path_ds(**kwargs):
    # Locate the node carrying isobaric temperature rather than hardcoding a
    # group path (mirrors the subtree walk in test_virtualizarr.py::tmp_paths).
    registry = ObjectStoreRegistry({"file://": LocalStore()})
    url = (TEST_DATA / FIXTURE).as_uri()
    store = GribberishParser(**kwargs)(url, registry)
    vdt = store.to_virtual_datatree()
    for node in vdt.subtree:
        ds = vdt[node.path].to_dataset()
        if "tmp" in ds.data_vars and any("isobar" in d for d in ds.dims):
            return store, node.path, ds
    raise AssertionError("no isobaric group found in fixture")


def _iso_ds(**kwargs):
    _, _, ds = _iso_store_path_ds(**kwargs)
    return ds


def _iso_dims(ds):
    return sorted(d for d in ds.dims if "isobar" in d)


def test_default_keeps_per_value_set_dims():
    ds = _iso_ds()
    # Three distinct isobaric value sets remain as separate suffixed dims.
    assert len(_iso_dims(ds)) == 3


def test_accumulate_produces_single_union_dim():
    base = _iso_ds()
    acc = _iso_ds(accumulate_dims=True)

    assert _iso_dims(acc) == ["isobar"]

    union = sorted(set().union(*[
        set(np.asarray(base[c].values).tolist())
        for c in base.coords if "isobar" in c
    ]))
    np.testing.assert_array_equal(np.asarray(acc["isobar"].values), union)
    assert acc.sizes["isobar"] == len(union)


def test_absent_level_reads_as_fill_and_present_level_reads_data():
    # `tmp` is backed by ManifestArray chunk references (real GRIB message
    # bytes, decoded by the gribberish codec at read time), which — unlike the
    # inlined `isobar` coordinate — cannot be materialized straight off the
    # virtual dataset (`.values` raises NotImplementedError: "ManifestArray
    # holds virtual references ... cannot be converted into a numpy array").
    # Read actual decoded chunks the same way test_virtualizarr.py does:
    # zarr.open(store, mode="r") on the concrete group path, indexed
    # positionally rather than via xarray's label-based `.sel`.
    base_store, base_path, base = _iso_store_path_ds()
    acc_store, acc_path, acc = _iso_store_path_ds(accumulate_dims=True)

    # tmp spans fewer isobaric levels than the union.
    tmp_dim = next(d for d in base["tmp"].dims if "isobar" in d)
    tmp_levels = np.asarray(base.coords[tmp_dim].values).tolist()
    union_levels = np.asarray(acc["isobar"].values).tolist()
    absent = sorted(set(union_levels) - set(tmp_levels))
    present = sorted(tmp_levels)

    assert absent, "expected tmp to be missing at least one union level"

    base_group = zarr.open(base_store, mode="r")[base_path.lstrip("/")]
    acc_group = zarr.open(acc_store, mode="r")[acc_path.lstrip("/")]

    # An absent level is entirely fill (NaN).
    absent_idx = union_levels.index(absent[0])
    miss = np.asarray(acc_group["tmp"][0, absent_idx])
    assert np.isnan(miss).all()

    # A present level matches the non-accumulated read at the same level.
    present_base_idx = tmp_levels.index(present[0])
    present_acc_idx = union_levels.index(present[0])
    want = np.asarray(base_group["tmp"][0, present_base_idx])
    got = np.asarray(acc_group["tmp"][0, present_acc_idx])
    assert not np.isnan(got).all()
    np.testing.assert_array_equal(got, want)


def test_real_height_above_ground_is_tagged_vertical():
    """The Rust fix: a real height-above-ground coordinate must carry axis='Z'
    end-to-end (this is the metadata the accumulation transform keys on)."""
    registry = ObjectStoreRegistry({"file://": LocalStore()})
    url = (TEST_DATA / "hrrr.t01z.wrfsfcf01-VVCSH-VUCSH.grib2").as_uri()
    store = GribberishParser()(url, registry)
    vdt = store.to_virtual_datatree()
    hag = None
    for node in vdt.subtree:
        ds = vdt[node.path].to_dataset()
        if "hag" in ds.coords:
            hag = ds["hag"]
            break
    assert hag is not None, "expected a hag coordinate in the fixture"
    assert hag.attrs.get("axis") == "Z"


def _find_ds_with_coord(store, coord_name):
    vdt = store.to_virtual_datatree()
    for node in vdt.subtree:
        ds = vdt[node.path].to_dataset()
        if coord_name in ds.coords:
            return ds
    return None


def test_percentile_coordinate_is_recognized_and_preserved():
    """Real percentile data: with accumulate_dims=True the percentile coord is
    recognized (accumulatable) and preserved. This fixture has a single
    percentile set, so accumulation is a no-op — but it must not be dropped,
    corrupted, or error."""
    registry = ObjectStoreRegistry({"file://": LocalStore()})
    url = (TEST_DATA / "s2s-pdt9-pdt10-pdt12.grib2").as_uri()
    base = _find_ds_with_coord(GribberishParser()(url, registry), "percentile")
    acc = _find_ds_with_coord(
        GribberishParser(accumulate_dims=True)(url, registry), "percentile"
    )
    assert base is not None and acc is not None
    np.testing.assert_array_equal(
        np.asarray(acc["percentile"].values), np.asarray(base["percentile"].values)
    )


def test_threshold_coordinate_is_recognized_and_preserved():
    registry = ObjectStoreRegistry({"file://": LocalStore()})
    url = (TEST_DATA / "nbm-pwat-prob-above.grib2").as_uri()
    base = _find_ds_with_coord(GribberishParser()(url, registry), "threshold")
    acc = _find_ds_with_coord(
        GribberishParser(accumulate_dims=True)(url, registry), "threshold"
    )
    assert base is not None and acc is not None
    np.testing.assert_array_equal(
        np.asarray(acc["threshold"].values), np.asarray(base["threshold"].values)
    )
