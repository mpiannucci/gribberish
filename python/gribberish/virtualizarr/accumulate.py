"""Merge per-value-set dimensions within each group into a single accumulated
dimension, and annotate each affected data variable with the mapping needed to
build its ManifestArray sparsely.

Accumulates every coordinate family that the Rust parser emits as one dimension
per distinct value set: vertical levels (identified by ``axis == "Z"``) and
``percentile`` / ``threshold`` coordinates (identified by ``standard_name``).

This is an opt-in post-processing step on the dict tree returned by
``parse_grib_dataset`` (see ``GribberishParser(accumulate_dims=True)``). It does
not touch Rust or the eager xarray backend.
"""

from __future__ import annotations

import re
from typing import Any

# A positional suffix appended by the Rust dim namer when a coordinate has more
# than one value set in a group (e.g. ``hag_0``, ``isobar_11``, ``percentile_1``).
_SUFFIX = re.compile(r"^(?P<base>.+?)_(?P<idx>\d+)$")

# Non-vertical coordinates that are still accumulatable, identified by their
# standard_name (they carry no ``axis`` attribute). Vertical levels are
# identified by ``axis == "Z"`` instead.
_ACCUMULATE_STANDARD_NAMES = frozenset({"percentile", "threshold"})


def _base_name(name: str) -> str:
    match = _SUFFIX.match(name)
    return match.group("base") if match else name


def _is_accumulatable(coord: dict[str, Any]) -> bool:
    attrs = coord.get("attrs", {})
    return (
        attrs.get("axis") == "Z"
        or attrs.get("standard_name") in _ACCUMULATE_STANDARD_NAMES
    )


def accumulate_dims(node: dict[str, Any]) -> dict[str, Any]:
    """Recursively accumulate per-value-set dimensions in ``node`` and its
    subgroups. Accumulates vertical levels (``axis == "Z"``) and percentile /
    threshold coordinates. Mutates and returns ``node``.
    """
    _accumulate_node(node)
    for child in node.get("groups", {}).values():
        accumulate_dims(child)
    return node


def _accumulate_node(node: dict[str, Any]) -> None:
    coords = node.get("coords", {})
    data_vars = node.get("data_vars", {})

    groups: dict[str, list[str]] = {}
    for name, coord in coords.items():
        if _is_accumulatable(coord):
            groups.setdefault(_base_name(name), []).append(name)

    for base, members in groups.items():
        if len(members) < 2:
            continue  # a single value set is already the plain name; nothing to do
        _accumulate_base(base, members, coords, data_vars)


def _accumulate_base(
    base: str,
    members: list[str],
    coords: dict[str, Any],
    data_vars: dict[str, Any],
) -> None:
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
        # A variable may be accumulated on more than one axis (e.g. a percentile
        # product that also spans multiple heights), so append rather than assign.
        var.setdefault("_accumulate", []).append(
            {"axis": vaxis, "index_map": [index_of[v] for v in own_values]}
        )

    attrs = dict(coords[members[0]]["attrs"])
    for member in members:
        del coords[member]
    coords[base] = {"dims": [base], "values": union, "attrs": attrs}
