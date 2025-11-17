import os
import fsspec

import numpy as np
import xarray as xr
from xarray.backends.common import BackendEntrypoint, BackendArray
from xarray.core import indexing

from gribberish import parse_grib_dataset, parse_grib_array, parse_grib_array_batch


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
        # other backend specific keyword arguments
    ):
        '''
        Open a GRIB2 file as an xarray Dataset with lazy loading.

        This implementation reads the file once to scan GRIB message metadata,
        then reads actual data from disk on demand when variables are accessed.
        This provides better performance and lower variance than caching.

        Parameters
        ----------
        filename_or_obj : str
            Path to the GRIB2 file
        '''
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
            )
            coords = {k: (v['dims'], v['values'], v['attrs']) for (k, v) in dataset['coords'].items()}

            # Create lazy backend arrays that read from disk on demand
            data_vars = {
                k: (
                    v['dims'],
                    GribberishBackendArray(
                        filename_or_obj,
                        storage_options=storage_options,
                        array_metadata=v['values']
                    ),
                    v['attrs']
                )
                for (k, v) in dataset['data_vars'].items()
            }
            attrs = dataset['attrs']

            return xr.Dataset(
                data_vars=data_vars,
                coords=coords,
                attrs=attrs
            )

    open_dataset_parameters = [
        "filename_or_obj",
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
        return ext in [".grib", ".grib2"]


class GribberishBackendArray(BackendArray):
    '''
    Custom backend array to support lazy loading of gribberish datasets.

    Data is read from disk on demand, providing consistent performance
    and lower memory footprint.
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
        # Thread-safe method that reads data from disk on demand
        arrs = []
        with self.lock:
            with fsspec.open(self.filename_or_obj, 'rb', **self.storage_options) as f:
                # Optimization: Check if messages are contiguous
                # If so, read all data at once and parse in batch
                if len(self.offsets) > 1:
                    # Check if contiguous (each message starts where previous ends)
                    is_contiguous = all(
                        self.offsets[i][0] + self.offsets[i][1] == self.offsets[i+1][0]
                        for i in range(len(self.offsets) - 1)
                    )

                    if is_contiguous:
                        # Read all messages in one shot
                        first_offset = self.offsets[0][0]
                        total_size = sum(size for _, size in self.offsets)
                        f.seek(first_offset, 0)
                        all_raw_data = f.read(total_size)

                        # Use batch parsing for better performance
                        # Calculate relative offsets from the start of the buffer
                        relative_offsets = []
                        current_pos = 0
                        for _, size in self.offsets:
                            relative_offsets.append(current_pos)
                            current_pos += size

                        arrs = parse_grib_array_batch(all_raw_data, relative_offsets)
                    else:
                        # Non-contiguous: fall back to individual reads
                        for offset, size in self.offsets:
                            f.seek(offset, 0)
                            raw_data = f.read(size)
                            chunk_data = parse_grib_array(raw_data, 0)
                            arrs.append(chunk_data)
                else:
                    # Single message: simple case
                    for offset, size in self.offsets:
                        f.seek(offset, 0)
                        raw_data = f.read(size)
                        chunk_data = parse_grib_array(raw_data, 0)
                        arrs.append(chunk_data)

        # Concatentate the flattened arrays, then reshape to the target shape
        data = np.concatenate(arrs)
        data = data.reshape(self.shape)

        # Return the applied index
        return data[key]