from gribberish.virtualizarr.accumulate import accumulate_dims


def _zcoord(values):
    return {"dims": [None], "values": list(values),
            "attrs": {"axis": "Z", "standard_name": "height", "long_name": "hag"}}


def _var(vdim, vlen):
    # A variable of shape (vertical, y, x) with one message per level.
    return {"dims": [vdim, "y", "x"], "attrs": {},
            "values": {"shape": [vlen, 2, 2],
                       "offsets": [(100 * (i + 1), 10) for i in range(vlen)]}}


def _node(coords, data_vars, groups=None):
    n = {"coords": dict(coords), "data_vars": dict(data_vars)}
    if groups is not None:
        n["groups"] = groups
    return n


def test_single_value_set_is_untouched():
    node = _node({"hag": _zcoord([2.0, 80.0])}, {"a": _var("hag", 2)})
    accumulate_dims(node)
    assert set(node["coords"]) == {"hag"}
    assert node["coords"]["hag"]["values"] == [2.0, 80.0]
    assert "_accumulate" not in node["data_vars"]["a"]
    assert node["data_vars"]["a"]["dims"] == ["hag", "y", "x"]


def test_two_value_sets_union_and_map():
    node = _node(
        {"hag_0": _zcoord([2.0, 80.0]), "hag_1": _zcoord([2.0, 10.0, 80.0])},
        {"a": _var("hag_0", 2), "b": _var("hag_1", 3)},
    )
    accumulate_dims(node)

    # Old suffixed coords are gone; a single union coord remains.
    assert set(node["coords"]) == {"hag"}
    assert node["coords"]["hag"]["values"] == [2.0, 10.0, 80.0]
    assert node["coords"]["hag"]["dims"] == ["hag"]
    assert node["coords"]["hag"]["attrs"]["axis"] == "Z"

    a = node["data_vars"]["a"]
    assert a["dims"] == ["hag", "y", "x"]
    assert a["values"]["shape"] == [3, 2, 2]          # vertical widened to union
    assert a["_accumulate"] == [{"axis": 0, "index_map": [0, 2]}]

    b = node["data_vars"]["b"]
    assert b["values"]["shape"] == [3, 2, 2]
    assert b["_accumulate"] == [{"axis": 0, "index_map": [0, 1, 2]}]


def test_independent_bases_accumulate_separately():
    node = _node(
        {"hag_0": _zcoord([2.0]), "hag_1": _zcoord([2.0, 10.0]),
         "isobar_0": _zcoord([500.0]), "isobar_1": _zcoord([500.0, 850.0])},
        {"h": _var("hag_1", 2), "i": _var("isobar_0", 1)},
    )
    accumulate_dims(node)
    assert node["coords"]["hag"]["values"] == [2.0, 10.0]
    assert node["coords"]["isobar"]["values"] == [500.0, 850.0]
    assert node["data_vars"]["i"]["_accumulate"] == [{"axis": 0, "index_map": [0]}]


def test_recurses_into_subgroups():
    child = _node({"hag_0": _zcoord([2.0]), "hag_1": _zcoord([2.0, 10.0])},
                  {"a": _var("hag_0", 1)})
    root = {"groups": {"hag": child}}
    accumulate_dims(root)
    assert set(child["coords"]) == {"hag"}
    assert child["data_vars"]["a"]["_accumulate"] == [{"axis": 0, "index_map": [0]}]


def _pcoord(values, std):
    return {"dims": [None], "values": list(values),
            "attrs": {"standard_name": std, "long_name": std}}


def test_percentile_is_accumulated_by_standard_name():
    node = _node(
        {"percentile": _pcoord([10, 50], "percentile"),
         "percentile_1": _pcoord([10, 50, 90], "percentile")},
        {"a": _var("percentile", 2), "b": _var("percentile_1", 3)},
    )
    accumulate_dims(node)
    assert set(node["coords"]) == {"percentile"}
    assert node["coords"]["percentile"]["values"] == [10, 50, 90]
    assert node["data_vars"]["a"]["_accumulate"] == [{"axis": 0, "index_map": [0, 1]}]
    assert node["data_vars"]["b"]["_accumulate"] == [{"axis": 0, "index_map": [0, 1, 2]}]


def test_threshold_is_accumulated_by_standard_name():
    node = _node(
        {"threshold": _pcoord([1.0, 5.0], "threshold"),
         "threshold_1": _pcoord([1.0, 5.0, 9.0], "threshold")},
        {"a": _var("threshold", 2)},
    )
    accumulate_dims(node)
    assert set(node["coords"]) == {"threshold"}
    assert node["coords"]["threshold"]["values"] == [1.0, 5.0, 9.0]
    assert node["data_vars"]["a"]["_accumulate"] == [{"axis": 0, "index_map": [0, 1]}]


def test_variable_accumulated_on_two_axes_gets_two_entries():
    # A var with BOTH a vertical (hag) and a percentile axis to merge.
    node = _node(
        {"hag_0": _zcoord([2.0]), "hag_1": _zcoord([2.0, 10.0]),
         "percentile": _pcoord([50], "percentile"),
         "percentile_1": _pcoord([50, 90], "percentile")},
        {"v": {"dims": ["hag_1", "percentile_1", "y", "x"], "attrs": {},
               "values": {"shape": [2, 2, 2, 2],
                          "offsets": [(100 * (i + 1), 10) for i in range(4)]}}},
    )
    accumulate_dims(node)
    v = node["data_vars"]["v"]
    assert v["dims"] == ["hag", "percentile", "y", "x"]
    assert v["values"]["shape"] == [2, 2, 2, 2]
    # one entry per accumulated axis (order not asserted); both present
    assert {"axis": 0, "index_map": [0, 1]} in v["_accumulate"]
    assert {"axis": 1, "index_map": [0, 1]} in v["_accumulate"]
    assert len(v["_accumulate"]) == 2


def test_time_and_number_are_not_accumulated():
    node = _node(
        {"time": {"dims": ["time"], "values": [0, 1],
                  "attrs": {"axis": "T", "standard_name": "time"}},
         "number": {"dims": ["number"], "values": [1, 2],
                    "attrs": {"standard_name": "realization"}},
         "number_1": {"dims": ["number_1"], "values": [1, 2, 3],
                      "attrs": {"standard_name": "realization"}}},
        {"a": _var("number_1", 3)},
    )
    accumulate_dims(node)
    # number_* left untouched (no axis=Z, standard_name not percentile/threshold)
    assert {"number", "number_1", "time"}.issubset(set(node["coords"]))
    assert "_accumulate" not in node["data_vars"]["a"]


import numpy as np
import pytest

pytest.importorskip("virtualizarr")

from virtualizarr.manifests import ChunkManifest
from gribberish.virtualizarr.parser import _data_manifest_array


def test_empty_path_cell_is_a_missing_chunk():
    # Establishes the sentinel the sparse builder relies on: a cell with an
    # empty-string path is absent from the manifest (and reads as fill_value).
    paths = np.array([["file:///f"], [""]], dtype=np.dtypes.StringDType())
    offsets = np.array([[0], [0]], dtype=np.uint64)
    lengths = np.array([[5], [0]], dtype=np.uint64)
    manifest = ChunkManifest.from_arrays(paths=paths, offsets=offsets, lengths=lengths)
    keys = set(manifest.dict().keys())
    assert "0.0" in keys
    assert "1.0" not in keys


def _accumulated_var(index_map, union_len):
    # Shape already widened to the union by the transform; one message per own level.
    return {
        "dims": ["hag", "latitude", "longitude"],
        "attrs": {},
        "values": {
            "shape": [union_len, 2, 2],
            "offsets": [(100 * (i + 1), 10) for i in range(len(index_map))],
        },
        "_accumulate": [{"axis": 0, "index_map": index_map}],
    }


def test_sparse_placement_leaves_absent_levels_missing():
    var = _accumulated_var(index_map=[0, 2], union_len=3)
    ma = _data_manifest_array("file:///x.grib2", "t", var)

    assert ma.metadata.shape == (3, 2, 2)
    keys = set(ma.manifest.dict().keys())
    assert keys == {"0.0.0", "2.0.0"}          # level 1 absent -> fill on read
    assert ma.manifest.dict()["2.0.0"]["offset"] == 200


def test_dense_variable_is_unchanged_without_annotation():
    var = {
        "dims": ["hag", "latitude", "longitude"],
        "attrs": {},
        "values": {"shape": [2, 2, 2], "offsets": [(100, 10), (200, 10)]},
    }
    ma = _data_manifest_array("file:///x.grib2", "t", var)
    assert set(ma.manifest.dict().keys()) == {"0.0.0", "1.0.0"}


def test_multi_axis_sparse_placement():
    # Vertical axis 0 (own len 2 -> union 3 at [0,2]) and percentile axis 1
    # (own len 2 -> union 3 at [1,2]); 2x2 = 4 messages in C order.
    var = {
        "dims": ["hag", "percentile", "latitude", "longitude"],
        "attrs": {},
        "values": {
            "shape": [3, 3, 2, 2],
            "offsets": [(10, 1), (20, 1), (30, 1), (40, 1)],
        },
        "_accumulate": [
            {"axis": 0, "index_map": [0, 2]},
            {"axis": 1, "index_map": [1, 2]},
        ],
    }
    ma = _data_manifest_array("file:///x.grib2", "p", var)
    assert ma.metadata.shape == (3, 3, 2, 2)
    keys = set(ma.manifest.dict().keys())
    # own (v,p) in {(0,0),(0,1),(1,0),(1,1)} -> union (0,1),(0,2),(2,1),(2,2)
    assert keys == {"0.1.0.0", "0.2.0.0", "2.1.0.0", "2.2.0.0"}


def test_sparse_guard_rejects_wrong_message_count():
    # index_map length 2 -> the variable's own grid expects 2 messages; supplying
    # 1 must raise rather than silently mis-placing chunks.
    var = {
        "dims": ["hag", "latitude", "longitude"],
        "attrs": {},
        "values": {"shape": [3, 2, 2], "offsets": [(100, 10)]},
        "_accumulate": [{"axis": 0, "index_map": [0, 2]}],
    }
    with pytest.raises(ValueError, match="expected 2 messages"):
        _data_manifest_array("file:///x.grib2", "t", var)
