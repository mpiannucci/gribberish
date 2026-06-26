"""Dataset structure must be a pure function of the file's schema.

Regression tests: coordinate suffixes (isobar_0/isobar_1/...) used to be
assigned by enumerating HashMap/HashSet iteration order, so which level set
got which name was randomized on every call. No data was lost, but a
variable's vertical dimension could be called isobar_0 on one open and
isobar_2 on the next — within a single process, and across files with an
identical schema, breaking multi-file workflows like xr.concat.
"""

import numpy as np
import pytest
from pathlib import Path

from gribberish import parse_grib_dataset, parse_grib_mapping

REPO_ROOT = Path(__file__).resolve().parents[2]
GEFS_FIXTURE = REPO_ROOT / "test-data" / "geavg.t12z.pgrb2a.0p50.f000"


def dataset_structure(node, with_offsets=True):
    """Reduce a dataset node tree to a comparable (coords, dims, groups) form."""
    coords = {}
    for name, coord in node.get("coords", {}).items():
        values = coord["values"]
        if isinstance(values, dict):
            values = tuple(sorted(values.get("offsets", []))) if with_offsets else None
        else:
            values = tuple(np.asarray(values).ravel().tolist())
        coords[name] = values
    dims = {var: tuple(dv["dims"]) for var, dv in node.get("data_vars", {}).items()}
    groups = {
        name: dataset_structure(group, with_offsets)
        for name, group in node.get("groups", {}).items()
    }
    return coords, dims, groups


def test_repeated_parses_yield_identical_structure():
    """Coordinate names, values and variable dims are stable across opens."""
    data = GEFS_FIXTURE.read_bytes()

    first = dataset_structure(parse_grib_dataset(data))
    for _ in range(9):
        assert dataset_structure(parse_grib_dataset(data)) == first


def test_message_order_does_not_affect_structure():
    """Two files with the same field set get identical dimension names even if
    their messages are stored in a different order, so datasets parsed from
    them can be joined."""
    data = GEFS_FIXTURE.read_bytes()

    offsets = sorted(offset for _, offset, _ in parse_grib_mapping(data).values())
    bounds = offsets + [len(data)]
    messages = [data[start:end] for start, end in zip(bounds, bounds[1:])]
    reversed_data = b"".join(reversed(messages))
    assert len(reversed_data) == len(data)

    # Byte offsets necessarily differ between the two layouts; everything
    # else (names, coordinate values, dims) must be identical.
    assert dataset_structure(
        parse_grib_dataset(reversed_data), with_offsets=False
    ) == dataset_structure(parse_grib_dataset(data), with_offsets=False)


def test_isobaric_level_sets_are_complete():
    """Every distinct isobaric level set in the file survives, with the level
    counts matching the messages actually present (cross-checked with wgrib2).
    Uses the collapsed layout so the isobaric hypercube sits directly under
    `/isobar`; the level sets are identical either way."""
    data = GEFS_FIXTURE.read_bytes()

    coords, dims, groups = dataset_structure(
        parse_grib_dataset(data, collapse_groups=True)
    )
    iso_coords, iso_dims, _ = groups["isobar"]

    isobar_sizes = sorted(
        len(values) for name, values in iso_coords.items() if "isobar" in name
    )
    assert isobar_sizes == [10, 11, 12]

    expected_levels = {"tmp": 10, "rh": 10, "hgt": 11, "ugrd": 12, "vgrd": 12}
    for var, expected in expected_levels.items():
        level_dim = next(d for d in iso_dims[var] if "isobar" in d)
        assert len(iso_coords[level_dim]) == expected, (
            f"{var}: expected {expected} levels, got "
            f"{len(iso_coords[level_dim])} on {level_dim}"
        )


def _group_paths(node, prefix=""):
    """All group paths that actually carry data variables."""
    out = []
    if node.get("data_vars"):
        out.append(prefix or "/")
    for name, group in node.get("groups", {}).items():
        out += _group_paths(group, prefix + "/" + name)
    return out


def test_stable_layout_group_path_is_content_independent():
    """The default layout pins each variable to a group path that depends only
    on its own metadata, so the same variable lands at the same path whether or
    not the file also carries it at other levels. The legacy collapse layout
    makes the path content-dependent, which is what broke multi-file concat
    (issue #165)."""
    SINGLE = REPO_ROOT / "test-data" / "hrrr.t06z.wrfsfcf01-TMP.grib2"  # tmp @ sfc
    MULTI = REPO_ROOT / "test-data" / "nbm-multilevel-tcdc.grib2"       # tmp @ hag+sfc

    single = _group_paths(parse_grib_dataset(SINGLE.read_bytes()))
    multi = _group_paths(parse_grib_dataset(MULTI.read_bytes()))
    # Surface tmp is at the same path in both, despite the multi-level file.
    assert "/sfc/instant" in single
    assert "/sfc/instant" in multi

    # Under the collapse layout the surface tmp's path is content-dependent.
    single_c = _group_paths(parse_grib_dataset(SINGLE.read_bytes(), collapse_groups=True))
    multi_c = _group_paths(parse_grib_dataset(MULTI.read_bytes(), collapse_groups=True))
    assert single_c == ["/"]
    assert "/sfc" in multi_c


def test_xarray_engine_dims_are_stable_across_opens():
    """The xarray engine exposes the same dim name per variable on every open."""
    xr = pytest.importorskip("xarray")

    def structure():
        ds = xr.open_dataset(
            str(GEFS_FIXTURE),
            engine="gribberish",
            group="isobar",
            collapse_groups=True,
        )
        try:
            return (
                {c: tuple(ds[c].values.tolist()) for c in ds.coords if "isobar" in c},
                {v: ds[v].dims for v in ds.data_vars},
            )
        finally:
            ds.close()

    first = structure()
    for _ in range(4):
        assert structure() == first
