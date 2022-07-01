import os 
import xarray as xr

import gribberish

from .gribberish import *
from xarray.backends import BackendEntrypoint


def read_binary_data(filename: str): 
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

        # TODO: Parse data into xarray format

        return None

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in [".grib", ".grib2"]