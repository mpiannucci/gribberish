"""Merge per-value-set vertical dimensions within each group into a single
accumulated vertical dimension, and annotate each affected data variable with
the mapping needed to build its ManifestArray sparsely.

This is an opt-in post-processing step on the dict tree returned by
``parse_grib_dataset`` (see ``GribberishParser(accumulate_dims=True)``). It does
not touch Rust or the eager xarray backend.
"""

from __future__ import annotations

import re
from typing import Any

# A positional suffix appended by the Rust dim namer when a coordinate has more
# than one value set in a group (e.g. ``hag_0``, ``isobar_11``).
_SUFFIX = re.compile(r"^(?P<base>.+?)_(?P<idx>\d+)$")


def _base_name(name: str) -> str:
    match = _SUFFIX.match(name)
    return match.group("base") if match else name


def accumulate_vertical_dims(node: dict[str, Any]) -> dict[str, Any]:
    """Recursively accumulate vertical dims in ``node`` and its subgroups.

    Mutates and returns ``node``.
    """
    _accumulate_node(node)
    for child in node.get("groups", {}).values():
        accumulate_vertical_dims(child)
    return node


def _accumulate_node(node: dict[str, Any]) -> None:
    coords = node.get("coords", {})
    data_vars = node.get("data_vars", {})

    # Group vertical coordinates by base name. Groups are split upstream by
    # coordinate_name, so under the default layout a node has a single base;
    # under collapse_groups a node may hold several, each accumulated on its own.
    verticals: dict[str, list[str]] = {}
    for name, coord in coords.items():
        if coord.get("attrs", {}).get("axis") == "Z":
            verticals.setdefault(_base_name(name), []).append(name)

    for base, members in verticals.items():
        if len(members) < 2:
            continue  # a single value set is already the plain name; nothing to do
        _accumulate_base(base, members, coords, data_vars)


def _accumulate_base(
    base: str,
    members: list[str],
    coords: dict[str, Any],
    data_vars: dict[str, Any],
) -> None:
    # Sorted (ascending) union of every member's values; each member's own
    # values are already ascending, matching the C order of its messages.
    union: list[Any] = sorted({v for m in members for v in coords[m]["values"]})
    index_of = {value: i for i, value in enumerate(union)}
    member_set = set(members)

    for var in data_vars.values():
        vdims = [d for d in var["dims"] if d in member_set]
        if not vdims:
            continue
        vname = vdims[0]
        vaxis = var["dims"].index(vname)
        own_values = coords[vname]["values"]

        var["dims"] = list(var["dims"])
        var["dims"][vaxis] = base
        shape = list(var["values"]["shape"])
        shape[vaxis] = len(union)
        var["values"]["shape"] = shape
        var["_accumulate"] = {
            "axis": vaxis,
            "index_map": [index_of[v] for v in own_values],
        }

    # Replace the suffixed coords with a single union coord under the base name,
    # inheriting the (identical) attrs of the members.
    attrs = dict(coords[members[0]]["attrs"])
    for member in members:
        del coords[member]
    coords[base] = {"dims": [base], "values": union, "attrs": attrs}
