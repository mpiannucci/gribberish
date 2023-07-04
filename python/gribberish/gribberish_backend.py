import os
import xarray as xr

from .gribberishpy import parse_grid_dataset
from xarray.backends import BackendEntrypoint


def read_binary_data(filename: str):
    # TODO: Make this streamed for large files, etc etc
    with open(filename, 'rb') as f:
        return f.read()


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
        # other backend specific keyword arguments
        # `chunks` and `cache` DO NOT go here, they are handled by xarray
    ):
        raw_data = read_binary_data(filename_or_obj)

        dataset = parse_grid_dataset(raw_data, drop_variables=drop_variables)
        coords = {k: (v['dims'], v['values'], v['attrs']) for (k, v) in dataset['coords'].items()}
        data_vars = {k: (v['dims'], v['values'], v['attrs']) for (k, v) in dataset['data_vars'].items()}
        attrs = dataset['attrs']

        return xr.Dataset(
            data_vars=data_vars,
            coords=coords,
            attrs=attrs
        )


    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in [".grib", ".grib2"]
