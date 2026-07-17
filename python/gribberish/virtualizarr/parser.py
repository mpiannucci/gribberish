"""VirtualiZarr parser for GRIB2 files, backed by gribberish.

Each GRIB message is a single chunk decoded at read time by the
``gribberish`` zarr codec. By default variables are split into nested groups by
surface type and product kind (a variable spanning multiple level types, or
instantaneous vs. accumulated/derived/probability products), mirroring the way
``cfgrib`` breaks a file into multiple datasets. This layout is
content-independent — a variable lands at the same path (e.g. ``/hag/instant``)
in every same-schema file, so multi-file datacubes concatenate cleanly. Pass
``collapse_groups=True`` to fold everything into one root dataset where possible,
with levels expressed as dimensions (cleaner per file, but the layout then
depends on the file's content).
"""

from __future__ import annotations

from typing import Any

import numpy as np

# Importing the codec registers "gribberish" in the zarr codec registry, which
# is what decodes each chunk at read time (and is required for zarr to validate
# the codec pipeline when the array metadata is constructed below).
from gribberish.zarr.codec import GribberishCodec  # noqa: F401
from gribberish import (
    adjust_latitude_values,
    adjust_longitude_values,
    parse_grib_dataset,
    parse_grib_dataset_from_headers,
)
from gribberish._index import (
    HEADER_BYTES,
    fetch_index_entries,
    get_ranges_batched,
    select_ranges,
)
from gribberish.virtualizarr.accumulate import accumulate_dims

from virtualizarr.manifests import (
    ChunkManifest,
    ManifestArray,
    ManifestGroup,
    ManifestStore,
)
from virtualizarr.manifests.manifest import INLINED_CHUNK_PATH
from virtualizarr.manifests.utils import create_v3_array_metadata

import obstore

__all__ = ["GribberishParser"]

_GRIBBERISH_CODEC = "gribberish"
_BYTES_CODEC = {"name": "bytes", "configuration": {"endian": "little"}}
# Number of trailing spatial dimensions on every variable (lat/lon or y/x).
_N_SPATIAL = 2


def _gribberish_codecs(
    var: str, *, adjust_longitude_range: bool = False, north_up: bool = False
) -> list[dict[str, Any]]:
    configuration: dict[str, Any] = {"var": var}
    if adjust_longitude_range:
        configuration["adjust_longitude_range"] = True
    if north_up:
        configuration["north_up"] = True
    return [{"name": _GRIBBERISH_CODEC, "configuration": configuration}]


def _data_manifest_array(
    url: str,
    name: str,
    var: dict[str, Any],
    *,
    adjust_longitude_range: bool = False,
    north_up: bool = False,
) -> ManifestArray:
    """One ManifestArray per data variable; each GRIB message is one chunk.

    When the variable was annotated by the accumulation transform
    (``var["_accumulate"]``, a list of per-axis mappings), its messages are
    placed at their mapped indices along each accumulated axis and the unfilled
    cells are left as missing chunks (empty path), which VirtualiZarr resolves
    to ``fill_value`` at read time.
    """
    dims = tuple(var["dims"])
    shape = tuple(int(s) for s in var["values"]["shape"])
    offsets_sizes = var["values"]["offsets"]

    # Each message is one chunk: spatial dims are a single chunk, every
    # non-spatial dim is chunked to length 1.
    chunk_shape = tuple([1] * (len(shape) - _N_SPATIAL) + list(shape[-_N_SPATIAL:]))
    grid_shape = tuple(list(shape[:-_N_SPATIAL]) + [1, 1])
    n_chunks = int(np.prod(grid_shape)) if grid_shape else 1

    paths = np.full(grid_shape, "", dtype=np.dtypes.StringDType())
    offsets = np.zeros(grid_shape, dtype=np.uint64)
    lengths = np.zeros(grid_shape, dtype=np.uint64)

    accumulate = var.get("_accumulate")
    if not accumulate:
        # Dense: Rust emits offsets pre-sorted in C order matching the dimension
        # order, so a flat C-order fill lines each message up with its cell.
        if len(offsets_sizes) != n_chunks:
            raise ValueError(
                f"variable {name!r}: expected {n_chunks} messages for shape "
                f"{shape} but got {len(offsets_sizes)}"
            )
        flat_paths = paths.reshape(-1)
        flat_offsets = offsets.reshape(-1)
        flat_lengths = lengths.reshape(-1)
        for i, (offset, size) in enumerate(offsets_sizes):
            flat_paths[i] = url
            flat_offsets[i] = offset
            flat_lengths[i] = size
    else:
        # Sparse: one or more axes were widened to their union. Messages remain
        # in C order over the variable's OWN (narrower) grid; unravel against
        # that grid and remap each accumulated axis into its union index.
        orig_grid = list(grid_shape)
        for acc in accumulate:
            orig_grid[acc["axis"]] = len(acc["index_map"])
        n_messages = int(np.prod(orig_grid))
        if len(offsets_sizes) != n_messages:
            raise ValueError(
                f"variable {name!r}: expected {n_messages} messages for its own "
                f"grid {tuple(orig_grid)} but got {len(offsets_sizes)}"
            )
        for i, (offset, size) in enumerate(offsets_sizes):
            index = list(np.unravel_index(i, orig_grid))
            for acc in accumulate:
                index[acc["axis"]] = acc["index_map"][index[acc["axis"]]]
            index = tuple(index)
            paths[index] = url
            offsets[index] = offset
            lengths[index] = size

    manifest = ChunkManifest.from_arrays(
        paths=paths, offsets=offsets, lengths=lengths
    )
    metadata = create_v3_array_metadata(
        shape=shape,
        data_type=np.dtype("float64"),
        chunk_shape=chunk_shape,
        fill_value=float("nan"),
        codecs=_gribberish_codecs(
            name,
            adjust_longitude_range=adjust_longitude_range,
            north_up=north_up,
        ),
        attributes={k: v for k, v in var["attrs"].items()},
        dimension_names=dims,
    )
    return ManifestArray(metadata=metadata, chunkmanifest=manifest)


def _reference_coord_array(
    url: str,
    name: str,
    coord: dict[str, Any],
    *,
    adjust_longitude_range: bool = False,
    north_up: bool = False,
) -> ManifestArray:
    """A coordinate stored as a byte range in the file (projected lat/lon),
    decoded by the gribberish codec at read time with the same adjustment
    flags as the data variables."""
    values = coord["values"]
    dims = tuple(coord["dims"])
    shape = tuple(int(s) for s in values["shape"])
    offset, size = values["offsets"][0]

    grid_shape = tuple([1] * len(shape))
    paths = np.array(url, dtype=np.dtypes.StringDType()).reshape(grid_shape)
    offsets = np.array(int(offset), dtype=np.uint64).reshape(grid_shape)
    lengths = np.array(int(size), dtype=np.uint64).reshape(grid_shape)

    manifest = ChunkManifest.from_arrays(
        paths=paths, offsets=offsets, lengths=lengths
    )
    metadata = create_v3_array_metadata(
        shape=shape,
        data_type=np.dtype("float64"),
        chunk_shape=shape,
        fill_value=float("nan"),
        codecs=_gribberish_codecs(
            name,
            adjust_longitude_range=adjust_longitude_range,
            north_up=north_up,
        ),
        attributes=dict(coord["attrs"]),
        dimension_names=dims,
    )
    return ManifestArray(metadata=metadata, chunkmanifest=manifest)


def _inline_coord_array(
    name: str,
    coord: dict[str, Any],
    *,
    adjust_longitude_range: bool = False,
    north_up: bool = False,
) -> ManifestArray:
    """A small derived coordinate (time/level/number/...) inlined as raw bytes."""
    dims = tuple(coord["dims"])
    attrs = dict(coord["attrs"])
    arr = np.asarray(coord["values"])

    # Wrap the 1-D longitude axis to match the roll the codec applies to each
    # data chunk. A projected grid's 2-D longitude is a reference, not inlined,
    # so the ndim guard never fires there.
    if adjust_longitude_range and name == "longitude" and arr.ndim == 1:
        arr = np.asarray(adjust_longitude_values(arr))

    # Flip the row-axis coordinate (axis "Y": `latitude` on regular grids,
    # projected `y` on Lambert) to north-first; a Lambert grid's 2-D `latitude`
    # is a reference flipped by the codec, so the ndim guard skips it here.
    if north_up and attrs.get("axis") == "Y" and arr.ndim == 1:
        arr = np.asarray(adjust_latitude_values(arr))

    if arr.dtype.kind == "M":
        # Store datetimes as CF-encoded int64 seconds so xarray can decode them.
        arr = arr.astype("datetime64[s]").astype("int64")
        attrs.setdefault("units", "seconds since 1970-01-01 00:00:00")
        attrs.setdefault("calendar", "proleptic_gregorian")

    # Capture the shape before making the buffer contiguous:
    # np.ascontiguousarray promotes 0-d arrays to ndim >= 1, which would turn a
    # scalar grid-mapping coordinate's () shape into (1,).
    shape = tuple(int(s) for s in arr.shape)
    data = np.ascontiguousarray(arr).tobytes()

    # One chunk covers the whole array, so the chunk grid mirrors the array's
    # dimensionality: () for a scalar (e.g. the grid-mapping coordinate), and
    # (1, 1, ...) otherwise. zarr requires the chunk grid and shape to share a
    # rank, so a scalar must stay 0-d rather than being padded to (1,).
    grid_shape = tuple([1] * len(shape))
    index = tuple([0] * len(shape))

    paths = np.full(grid_shape, INLINED_CHUNK_PATH, dtype=np.dtypes.StringDType())
    offsets = np.zeros(grid_shape, dtype=np.uint64)
    lengths = np.full(grid_shape, arr.nbytes, dtype=np.uint64)

    manifest = ChunkManifest.from_arrays(
        paths=paths,
        offsets=offsets,
        lengths=lengths,
        inlined={index: data},
    )
    metadata = create_v3_array_metadata(
        shape=shape,
        data_type=arr.dtype,
        chunk_shape=shape,
        fill_value=None,
        codecs=[_BYTES_CODEC],
        attributes=attrs,
        dimension_names=dims,
    )
    return ManifestArray(metadata=metadata, chunkmanifest=manifest)


def _coord_manifest_array(
    url: str,
    name: str,
    coord: dict[str, Any],
    *,
    adjust_longitude_range: bool = False,
    north_up: bool = False,
) -> ManifestArray:
    if isinstance(coord["values"], dict):
        return _reference_coord_array(
            url,
            name,
            coord,
            adjust_longitude_range=adjust_longitude_range,
            north_up=north_up,
        )
    return _inline_coord_array(
        name,
        coord,
        adjust_longitude_range=adjust_longitude_range,
        north_up=north_up,
    )


def _manifest_group(
    url: str,
    node: dict[str, Any],
    *,
    adjust_longitude_range: bool = False,
    north_up: bool = False,
) -> ManifestGroup:
    """Recursively build a ManifestGroup (and its subgroups) from a tree node."""
    arrays: dict[str, ManifestArray] = {}
    coord_names: list[str] = []

    for name, coord in node.get("coords", {}).items():
        arrays[name] = _coord_manifest_array(
            url,
            name,
            coord,
            adjust_longitude_range=adjust_longitude_range,
            north_up=north_up,
        )
        coord_names.append(name)

    for name, var in node.get("data_vars", {}).items():
        arrays[name] = _data_manifest_array(
            url,
            name,
            var,
            adjust_longitude_range=adjust_longitude_range,
            north_up=north_up,
        )

    groups = {
        gname: _manifest_group(
            url,
            gnode,
            adjust_longitude_range=adjust_longitude_range,
            north_up=north_up,
        )
        for gname, gnode in node.get("groups", {}).items()
    }

    attributes = dict(node.get("attrs", {}))
    if coord_names:
        # Tell xarray which arrays are coordinates rather than data variables.
        attributes["coordinates"] = " ".join(coord_names)

    return ManifestGroup(arrays=arrays, groups=groups, attributes=attributes)


def _read_all(store, path: str) -> bytes:
    return obstore.open_reader(store, path).read().to_bytes()


class GribberishParser:
    """A VirtualiZarr parser that turns a GRIB2 file into a ``ManifestStore``.

    Parameters
    ----------
    drop_variables
        Variable short names to ignore.
    only_variables
        If given, only these variable short names are kept.
    perserve_dims
        Dimension/level-type names to keep even when their length is 1. Combine
        with ``accumulate_dims`` to surface single-level variables onto the
        shared accumulated axis (see ``accumulate_dims``).
    filter_by_attrs
        Keep only variables whose attributes match these values.
    filter_by_variable_attrs
        Per-variable attribute filter (takes precedence over ``filter_by_attrs``).
    use_index
        Build the manifest from a sidecar index (NOAA wgrib2 ``.idx`` or ECMWF
        open-data ``.index``) instead of downloading the whole GRIB file: the
        index locates every message, and only each message's leading header
        bytes are fetched (by range) for metadata — data sections are never
        read, since manifest chunks point back at the file. ``"auto"`` probes
        the known index names and silently falls back to a full read when none
        is found; an explicit suffix (``".idx"``, ``".index"``, ``".inv"``,
        ...) probes only that name and raises when it is missing.
    adjust_longitude_range
        Rewrap global 0–360° longitude grids to a monotonic −180…180° range:
        the emitted ``longitude`` coordinate is wrapped and every data variable's
        codec is told to roll its decoded chunk along the longitude axis to
        match, so the published store slices cleanly across the prime meridian.
        Default off; a no-op for grids that don't span the globe.
    north_up
        Reorder every grid so the 0th row is the northern-most: the emitted
        ``latitude`` coordinate is flipped and every data variable's codec is told
        to reverse its decoded chunk's rows to match. Default off; a no-op for
        grids that are already north-first.
    collapse_groups
        Default off, which gives a **stable, content-independent** group layout:
        every variable is nested under its surface-type and product-kind
        subgroups (e.g. ``/hag/instant``) regardless of whether anything in the
        file conflicts, so a variable's group path depends only on its own
        metadata and is identical across every file in a forecast sequence —
        letting multi-file datacubes concatenate cleanly. Turn it **on** to
        collapse everything into one root dataset where possible (levels and
        kinds become dimensions, and a subgroup only appears when a variable in
        *this* file actually spans more than one of its values). That is cleaner
        for a single file but makes the layout content-dependent: the same
        variable can land at different paths across files (``/hag/instant`` in
        one, ``/instant`` in another), which breaks concatenation.
    accumulate_dims
        Default off. When on, per-value-set dimensions within a group — vertical
        levels (``hag_0``, ``hag_1``, …), and ``percentile`` / ``threshold`` —
        are merged into a single accumulated dimension whose coordinate is the
        sorted union of every value present, and each variable is made sparse
        over that axis: it carries chunk references only for the values it
        actually has, and absent slots read back as the fill value (``NaN``).
        This gives one shared, schema-agnostic coordinate per group, so files
        with differing value sets align cleanly under ``join="outer"``.
        VirtualiZarr path only.

        Only dimensions that survive parsing are accumulated. A variable present
        at a single value of any accumulatable family — one vertical level (e.g.
        10 m wind), one percentile, or one threshold — normally has that length-1
        dimension dropped, so it does not appear on the accumulated axis. Pass
        the corresponding name(s) in ``perserve_dims`` (e.g. ``["hag"]``,
        ``["percentile"]``, ``["threshold"]``) alongside ``accumulate_dims`` to
        keep those single-value dimensions: the variables then join the shared
        axis sparsely — real data at their one value, ``NaN`` elsewhere,
        selectable by label, e.g. ``ds["wind"].sel(hag=10.0)``.
    """

    def __init__(
        self,
        drop_variables: list[str] | None = None,
        only_variables: list[str] | None = None,
        perserve_dims: list[str] | None = None,
        filter_by_attrs: dict[str, Any] | None = None,
        filter_by_variable_attrs: dict[str, Any] | None = None,
        use_index: bool | str = False,
        adjust_longitude_range: bool = False,
        north_up: bool = False,
        collapse_groups: bool = False,
        accumulate_dims: bool = False,
    ) -> None:
        self.drop_variables = drop_variables
        self.only_variables = only_variables
        self.perserve_dims = perserve_dims
        self.filter_by_attrs = filter_by_attrs
        self.filter_by_variable_attrs = filter_by_variable_attrs
        self.use_index = use_index
        self.adjust_longitude_range = adjust_longitude_range
        self.north_up = north_up
        self.collapse_groups = collapse_groups
        self.accumulate_dims = accumulate_dims

    def _filter_kwargs(self) -> dict[str, Any]:
        return dict(
            drop_variables=self.drop_variables,
            only_variables=self.only_variables,
            perserve_dims=self.perserve_dims,
            filter_by_attrs=self.filter_by_attrs,
            filter_by_variable_attrs=self.filter_by_variable_attrs,
            # Keep projected lat/lon as references rather than materializing them.
            encode_coords=True,
            collapse_groups=self.collapse_groups,
        )

    def _parse_via_index(self, store, path: str, entries) -> dict[str, Any]:
        ranges = select_ranges(entries, self.only_variables, self.drop_variables)
        starts, sizes = list(ranges), list(ranges.values())

        # Header windows carry all the metadata (sections 0-5). A window can
        # fall short (oversized grid definition, GRIB1) — retry with the whole
        # messages, still only the ones the filters keep.
        for fetch_lengths in ([min(s, HEADER_BYTES) for s in sizes], sizes):
            # Small coalesce: merging only (near-)contiguous windows, so the
            # data sections we're skipping never get transferred.
            chunks = get_ranges_batched(
                store, path, starts, fetch_lengths, coalesce=HEADER_BYTES
            )
            messages = [
                (offset, size, bytes(chunk))
                for offset, size, chunk in zip(starts, sizes, chunks)
            ]
            try:
                return parse_grib_dataset_from_headers(
                    messages, **self._filter_kwargs()
                )
            except ValueError as err:
                if "message header" not in str(err):
                    raise
        raise ValueError(f"failed to parse messages of {path!r} located by its index")

    def __call__(self, url: str, registry) -> ManifestStore:
        store, path_in_store = registry.resolve(url)

        dataset = None
        if self.use_index:
            # Missing index (FileNotFoundError) or unparseable index
            # (ValueError) — "auto" falls back to reading the whole file.
            try:
                entries = fetch_index_entries(store, path_in_store, self.use_index)
            except (FileNotFoundError, ValueError):
                if self.use_index != "auto":
                    raise
            else:
                dataset = self._parse_via_index(store, path_in_store, entries)

        if dataset is None:
            data = _read_all(store, path_in_store)
            dataset = parse_grib_dataset(data, **self._filter_kwargs())

        if self.accumulate_dims:
            dataset = accumulate_dims(dataset)

        group = _manifest_group(
            url,
            dataset,
            adjust_longitude_range=self.adjust_longitude_range,
            north_up=self.north_up,
        )
        return ManifestStore(group, registry=registry)
