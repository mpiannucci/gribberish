import os
from pathlib import Path

import numpy as np
import obstore
import xarray as xr
from obstore.store import from_url
from xarray.backends.common import BackendEntrypoint, BackendArray
from xarray.core import indexing

from gribberish import parse_grib_dataset, parse_grib_array


DATA_VAR_LOCK = xr.backends.locks.SerializableLock()


def _store_and_path(filename_or_obj, storage_options):
    """Resolve a path or URL to an obstore store rooted at its parent and the
    object name within it.

    A bare local path becomes a ``file://`` URI; remote URLs (``s3://``,
    ``gs://``, ``https://`` …) are passed through. ``storage_options`` are
    forwarded to :func:`obstore.store.from_url` as backend configuration.
    """
    path = os.fspath(filename_or_obj)
    if "://" not in path:
        path = Path(path).resolve().as_uri()
    base, _, name = path.rpartition("/")
    return from_url(base, **storage_options), name


def _iter_leaf_groups(node, prefix=""):
    """Yield the path of every node that actually holds data variables."""
    if node.get("data_vars"):
        yield prefix or "/"
    for name, child in node.get("groups", {}).items():
        child_prefix = f"{prefix}/{name}" if prefix else f"/{name}"
        yield from _iter_leaf_groups(child, child_prefix)


def _iter_all_nodes(node, prefix="/"):
    """Yield (path, node) for the root and every descendant group."""
    yield prefix, node
    for name, child in node.get("groups", {}).items():
        child_prefix = f"{prefix.rstrip('/')}/{name}"
        yield from _iter_all_nodes(child, child_prefix)


def _navigate(tree, group):
    """Descend the tree to the named group (e.g. 'isobar' or 'sfc/accum')."""
    node = tree
    for segment in group.strip("/").split("/"):
        if not segment:
            continue
        node = node.get("groups", {}).get(segment)
        if node is None:
            return None
    return node


def _node_to_dataset(node, filename_or_obj, storage_options):
    coords = {
        name: (coord["dims"], coord["values"], coord["attrs"])
        for name, coord in node.get("coords", {}).items()
    }
    data_vars = {
        name: (
            var["dims"],
            GribberishBackendArray(
                filename_or_obj,
                storage_options=storage_options,
                array_metadata=var["values"],
            ),
            var["attrs"],
        )
        for name, var in node.get("data_vars", {}).items()
    }
    return xr.Dataset(
        data_vars=data_vars, coords=coords, attrs=node.get("attrs", {})
    )


def _group_error(tree, requested=None):
    available = "\n  - ".join(_iter_leaf_groups(tree))
    if requested is None:
        return (
            "This GRIB file maps to multiple groups (conflicting hypercubes). "
            "Pass group=<name> to open_dataset, or use xarray.open_datatree() to "
            f"open all groups at once.\nAvailable groups:\n  - {available}"
        )
    return f"Group {requested!r} not found.\nAvailable groups:\n  - {available}"


class GribberishBackend(BackendEntrypoint):
    supports_groups = True

    '''
    Custom backend for xarray

    Adapted from https://xarray.pydata.org/en/stable/internals/how-to-add-new-backend.html

    Conflicting hypercubes (a variable at multiple level types, or
    instantaneous vs. accumulated/derived/probability products) are exposed as
    separate groups, mirroring the way ``cfgrib`` breaks a file into multiple
    datasets. Use ``group=`` to select one, or ``xarray.open_datatree`` /
    ``xarray.open_groups`` to get them all. A conflict-free file opens directly.
    '''

    def _parse(self, filename_or_obj, storage_options, drop_variables,
               only_variables, perserve_dims, filter_by_attrs,
               filter_by_variable_attrs):
        store, path = _store_and_path(filename_or_obj, storage_options)
        raw_data = obstore.get(store, path).bytes().to_bytes()
        return parse_grib_dataset(
            raw_data,
            drop_variables=drop_variables,
            only_variables=only_variables,
            perserve_dims=perserve_dims,
            filter_by_attrs=filter_by_attrs,
            filter_by_variable_attrs=filter_by_variable_attrs,
        )

    def open_dataset(
        self,
        filename_or_obj,
        *,
        drop_variables=None,
        group=None,
        storage_options=None,
        only_variables=None,
        perserve_dims=None,
        filter_by_attrs=None,
        filter_by_variable_attrs=None,
    ):
        storage_options = storage_options or {}
        tree = self._parse(
            filename_or_obj, storage_options, drop_variables, only_variables,
            perserve_dims, filter_by_attrs, filter_by_variable_attrs,
        )

        has_groups = bool(tree.get("groups"))
        if group in (None, "", "/"):
            if has_groups:
                raise ValueError(_group_error(tree))
            node = tree
        else:
            node = _navigate(tree, group)
            if node is None:
                raise ValueError(_group_error(tree, requested=group))

        return _node_to_dataset(node, filename_or_obj, storage_options)

    def open_groups_as_dict(
        self,
        filename_or_obj,
        *,
        drop_variables=None,
        storage_options=None,
        only_variables=None,
        perserve_dims=None,
        filter_by_attrs=None,
        filter_by_variable_attrs=None,
    ):
        storage_options = storage_options or {}
        tree = self._parse(
            filename_or_obj, storage_options, drop_variables, only_variables,
            perserve_dims, filter_by_attrs, filter_by_variable_attrs,
        )
        return {
            path: _node_to_dataset(node, filename_or_obj, storage_options)
            for path, node in _iter_all_nodes(tree)
        }

    def open_datatree(self, filename_or_obj, **kwargs):
        return xr.DataTree.from_dict(
            self.open_groups_as_dict(filename_or_obj, **kwargs)
        )

    open_dataset_parameters = [
        "filename_or_obj",
        "group",
        "drop_variables",
        "only_variables",
        "perserve_dims",
        "filter_by_attrs",
        "filter_by_variable_attrs",
        "storage_options",
    ]

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext.lower() in [".grib", ".grib1", ".grib2"]


class GribberishBackendArray(BackendArray):
    '''
    Custom backend array to support lazy loading of gribberish datasets
    '''

    def __init__(
        self,
        filename_or_obj,
        array_metadata,
        storage_options=None,
        # other backend specific keyword arguments
    ):
        self.filename_or_obj = filename_or_obj
        self.storage_options = storage_options or {}
        self.shape = array_metadata['shape']
        self.offsets = array_metadata['offsets']
        self.dtype = np.dtype(np.float64)
        self.lock = DATA_VAR_LOCK

        # For now, we rely on the builtin indexing support but explicitely
        # set the indexers to be the array itself to utilize the same __getitem__ method
        self.oindex = self
        self.vindex = self

    def __getitem__(
        self, key: xr.core.indexing.ExplicitIndexer
    ) -> np.typing.ArrayLike:
        return indexing.explicit_indexing_adapter(
            key,
            self.shape,
            indexing.IndexingSupport.OUTER_1VECTOR,
            self._raw_indexing_method,
        )

    def _raw_indexing_method(self, key: tuple) -> np.typing.ArrayLike:
        # thread safe method that access to data on disk
        with self.lock:
            store, path = _store_and_path(self.filename_or_obj, self.storage_options)
            # One ranged read per GRIB message; obstore fetches them in
            # parallel and coalesces nearby ranges into single requests.
            chunks = obstore.get_ranges(
                store,
                path,
                starts=[offset for offset, _ in self.offsets],
                lengths=[size for _, size in self.offsets],
            )

        # Each chunk is the raw bytes of one message; decode the spatial slab.
        arrs = [parse_grib_array(bytes(chunk), 0) for chunk in chunks]

        # Concatentate the flattened arrays, the reshape to the target shape
        data = np.concatenate(arrs)
        data = data.reshape(self.shape)

        # Return the applied index
        return data[key]
