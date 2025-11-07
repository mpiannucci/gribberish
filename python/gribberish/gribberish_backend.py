import os
import fsspec

import numpy as np
import xarray as xr
from xarray.backends.common import BackendEntrypoint, BackendArray
from xarray.core import indexing

from gribberish import parse_grib_dataset, parse_grib_array


DATA_VAR_LOCK = xr.backends.locks.SerializableLock()


class GribberishBackend(BackendEntrypoint):
    '''
    Custom backend for xarray

    Adapted from https://xarray.pydata.org/en/stable/internals/how-to-add-new-backend.html
    '''

    def open_dataset(
        self,
        filename_or_obj,
        storage_options=None,
        drop_variables=None,
        only_variables=None,
        perserve_dims=None,
        filter_by_attrs=None,
        filter_by_variable_attrs=None,
        cfgrib_compat=False,
    ):
        storage_options = storage_options or {}

        with fsspec.open(filename_or_obj, 'rb', **storage_options) as f:
            raw_data = f.read()

            dataset =  parse_grib_dataset(
                raw_data,
                drop_variables=drop_variables,
                only_variables=only_variables,
                perserve_dims=perserve_dims,
                filter_by_attrs=filter_by_attrs,
                filter_by_variable_attrs=filter_by_variable_attrs,
                cfgrib_compat=cfgrib_compat,
            )
            coords = {k: (v['dims'], v['values'], v['attrs']) for (k, v) in dataset['coords'].items()}

            if cfgrib_compat:
                coords = {k: v for k, v in coords.items() if k not in ['x', 'y']}

            data_vars = {k: (v['dims'], GribberishBackendArray(filename_or_obj, storage_options=storage_options, array_metadata=v['values']) , v['attrs']) for (k, v) in dataset['data_vars'].items()}
            attrs = dataset['attrs']

            ds = xr.Dataset(
                data_vars=data_vars,
                coords=coords,
                attrs=attrs
            )

            if cfgrib_compat:
                # Squeeze out dimensions with size 1 (time, level, etc.)
                # drop=False keeps them as scalar coordinates
                ds = ds.squeeze(drop=False)

            return ds

    open_dataset_parameters = [
        "filename_or_obj",
        "drop_variables",
        "only_variables",
        "perserve_dims",
        "filter_by_attrs",
        "filter_by_variable_attrs",
        "storage_options",
        "cfgrib_compat",
    ]

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in [".grib", ".grib2"]


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
        arrs = []
        with self.lock:
            with fsspec.open(self.filename_or_obj, 'rb', **self.storage_options) as f:
                for offset, size in self.offsets:
                    f.seek(offset, 0)
                    raw_data = f.read(size)

                    # Current offset is the beginning of the raw data chunk
                    # The shape is the shape of the spatial portion of the 
                    # data chunk
                    chunk_data = parse_grib_array(raw_data, 0)
                    arrs.append(chunk_data)
    
        # Concatentate the flattened arrays, the reshape to the target shape
        data = np.concatenate(arrs)
        data = data.reshape(self.shape)

        # Return the applied index
        return data[key]
