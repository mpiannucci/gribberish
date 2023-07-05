import os
import xarray as xr
import numpy as np

from .gribberishpy import parse_grid_dataset, build_grib_array
from xarray.backends.common import BackendEntrypoint, BackendArray, AbstractDataStore
from xarray.core import indexing


def read_binary_data(filename: str):
    # TODO: Make this streamed for large files, etc etc
    with open(filename, 'rb') as f:
        return f.read()


DATA_VAR_LOCK = xr.backends.locks.SerializableLock()


class GribberishBackend(BackendEntrypoint):
    '''
    Custom backend for xarray

    Adapted from https://xarray.pydata.org/en/stable/internals/how-to-add-new-backend.html
    '''

    def open_dataset(
        self,
        filename_or_obj,
        *,
        drop_variables=None,
        only_variables=None,
        # other backend specific keyword arguments
        # `chunks` and `cache` DO NOT go here, they are handled by xarray
    ):
        with open(filename_or_obj, 'rb') as f:
            raw_data = f.read()

            dataset = parse_grid_dataset(raw_data, drop_variables=drop_variables, only_variables=only_variables)
            coords = {k: (v['dims'], v['values'], v['attrs']) for (k, v) in dataset['coords'].items()}
            data_vars = {k: (v['dims'], GribberishBackendArray(filename_or_obj, array_metadata=v['values']) , v['attrs']) for (k, v) in dataset['data_vars'].items()}
            attrs = dataset['attrs']

            return xr.Dataset(
                data_vars=data_vars,
                coords=coords,
                attrs=attrs
            )

    open_dataset_parameters = ["filename_or_obj", "drop_variables", "only_variables"]

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in [".grib", ".grib2"]


class GribberishBackendArray(BackendArray):
    '''
    Custom backend array for xarray to support lazy loading of gribberish datasets
    '''
    def __init__(
        self,
        filename_or_obj,
        array_metadata,
        # other backend specific keyword arguments
    ):
        self.filename_or_obj = filename_or_obj
        self.shape = array_metadata['shape']
        self.offsets = array_metadata['offsets']
        self.dtype = np.dtype(np.float64)
        self.lock = DATA_VAR_LOCK

    def __getitem__(
        self, key: xr.core.indexing.ExplicitIndexer
    ) -> np.typing.ArrayLike:
        return indexing.explicit_indexing_adapter(
            key,
            self.shape,
            indexing.IndexingSupport.BASIC,
            self._raw_indexing_method,
        )

    def _raw_indexing_method(self, key: tuple) -> np.typing.ArrayLike:
        # thread safe method that access to data on disk
        with self.lock:
            with open(self.filename_or_obj, 'rb') as f:
                raw_data = f.read()
                data = build_grib_array(raw_data, self.shape, self.offsets)

        return data[key]