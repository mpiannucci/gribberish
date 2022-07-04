import os
import numpy as np
import xarray as xr
import rasterio.crs as rio_crs

from .gribberish import parse_grib_mapping, parse_grib_message
from xarray.backends import BackendEntrypoint


def read_binary_data(filename: str):
    with open(filename, 'rb') as f:
        return f.read()


def extract_variable_data(grib_message):
    data = np.expand_dims(grib_message.data(), axis=0)

    return (
        ['time', 'lat', 'lon'],
        data, 
        {
            'standard_name': grib_message.var_abbrev, 
            'long_name': grib_message.var_name,
            'units': grib_message.units, 
        }
    )


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

        # Read the message mapping from the metadata that gives the byte offset for
        # each variables message
        var_mapping = parse_grib_mapping(raw_data)

        # If there are variabels specified to drop, do so now
        if drop_variables:
            for var in drop_variables:
                var_mapping.pop(var, None)

        # Extract each variables metadata
        data_vars = {var: extract_variable_data(parse_grib_message(
            raw_data, lookup[1])) for (var, lookup) in var_mapping.items()}

        # Get the coordinate arrays
        # TODO: This can be optimized
        first_message = parse_grib_message(raw_data, list(var_mapping.values())[0][1])
        coords = {
            'time': (['time'], [first_message.forecast_date], {
                'standard_name': 'time', 
                'long_name': 'time',
                'units': 'seconds since 2010-01-01 00:00:00',
                'axis': 'T'
            }),
            'lat': (['lat'], first_message.latitudes(), {
                'standard_name': 'latitude',
                'long_name': 'latitude',
                'units': 'degrees_north', 
                'axis': 'Y'
            }), 
            'lon': (['lon'], first_message.longitudes(), {
                'standard_name': 'longitude', 
                'long_name': 'longitude',
                'units': 'degrees_east', 
                'axis': 'X'
            }),
        }

        crs = rio_crs.CRS.from_proj4(first_message.proj)

        # Finally put it all together and create the xarray dataset
        return xr.Dataset(
            data_vars=data_vars,
            coords=coords,
            attrs={
                'meta': 'created with gribberish',
                'crs': crs,
            }
        )

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in [".grib", ".grib2"]
