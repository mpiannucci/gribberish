from gribberish.virtualizarr.accumulate import accumulate_vertical_dims


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
    accumulate_vertical_dims(node)
    assert set(node["coords"]) == {"hag"}
    assert node["coords"]["hag"]["values"] == [2.0, 80.0]
    assert "_accumulate" not in node["data_vars"]["a"]
    assert node["data_vars"]["a"]["dims"] == ["hag", "y", "x"]


def test_two_value_sets_union_and_map():
    node = _node(
        {"hag_0": _zcoord([2.0, 80.0]), "hag_1": _zcoord([2.0, 10.0, 80.0])},
        {"a": _var("hag_0", 2), "b": _var("hag_1", 3)},
    )
    accumulate_vertical_dims(node)

    # Old suffixed coords are gone; a single union coord remains.
    assert set(node["coords"]) == {"hag"}
    assert node["coords"]["hag"]["values"] == [2.0, 10.0, 80.0]
    assert node["coords"]["hag"]["dims"] == ["hag"]
    assert node["coords"]["hag"]["attrs"]["axis"] == "Z"

    a = node["data_vars"]["a"]
    assert a["dims"] == ["hag", "y", "x"]
    assert a["values"]["shape"] == [3, 2, 2]          # vertical widened to union
    assert a["_accumulate"] == {"axis": 0, "index_map": [0, 2]}

    b = node["data_vars"]["b"]
    assert b["values"]["shape"] == [3, 2, 2]
    assert b["_accumulate"] == {"axis": 0, "index_map": [0, 1, 2]}


def test_independent_bases_accumulate_separately():
    node = _node(
        {"hag_0": _zcoord([2.0]), "hag_1": _zcoord([2.0, 10.0]),
         "isobar_0": _zcoord([500.0]), "isobar_1": _zcoord([500.0, 850.0])},
        {"h": _var("hag_1", 2), "i": _var("isobar_0", 1)},
    )
    accumulate_vertical_dims(node)
    assert node["coords"]["hag"]["values"] == [2.0, 10.0]
    assert node["coords"]["isobar"]["values"] == [500.0, 850.0]
    assert node["data_vars"]["i"]["_accumulate"] == {"axis": 0, "index_map": [0]}


def test_recurses_into_subgroups():
    child = _node({"hag_0": _zcoord([2.0]), "hag_1": _zcoord([2.0, 10.0])},
                  {"a": _var("hag_0", 1)})
    root = {"groups": {"hag": child}}
    accumulate_vertical_dims(root)
    assert set(child["coords"]) == {"hag"}
    assert child["data_vars"]["a"]["_accumulate"] == {"axis": 0, "index_map": [0]}
