import os
import numpy as np
import xarray as xr

from .gribberishpy import parse_grib_mapping, parse_grib_message
from xarray.backends import BackendEntrypoint


def read_binary_data(filename: str):
    with open(filename, 'rb') as f:
        return f.read()


def extract_variable_data(grib_message):
    data = grib_message.data().reshape(grib_message.grid_shape)
    data = np.expand_dims(data, axis=0)
    crs = grib_message.crs
    return (
        ['time', 'lat', 'lon'] if grib_message.is_regular_grid else ['time', 'y', 'x'],
        data,
        {
            'standard_name': grib_message.var_abbrev,
            'long_name': grib_message.var_name,
            'units': grib_message.units,
            'crs': crs,
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

        # Read the message mapping from the metadata that gives the byte offset
        # for each variables message
        var_mapping = parse_grib_mapping(raw_data)

        # For now, any unsupported products get dropped 
        keys = list(var_mapping.keys())
        for var in keys: 
            if var.startswith('(') or var == 'unknown': 
                var_mapping.pop(var, None)

        # If there are variabels specified to drop, do so now
        if drop_variables:
            for var in drop_variables:
                var_mapping.pop(var, None)

        # Extract each variables metadata
        data_vars = {var: extract_variable_data(parse_grib_message(
            raw_data, lookup[1])) for (var, lookup) in var_mapping.items()}

        # Get the coordinate arrays
        # TODO: This can be optimized
        first_message = parse_grib_message(
            raw_data,
            list(var_mapping.values())[0][1]
        )

        lat = first_message.latitudes().reshape(first_message.grid_shape)
        lng = first_message.longitudes().reshape(first_message.grid_shape)

        if first_message.is_regular_grid:
            lat = (['lat'], lat[:, 0], {
                'standard_name': 'latitude',
                'long_name': 'latitude',
                'units': 'degrees_north',
                'axis': 'Y'
            })
            lon = (['lon'], lng[0, :], {
                'standard_name': 'longitude',
                'long_name': 'longitude',
                'units': 'degrees_east',
                'axis': 'X'
            })
        else:
            lat = (['y', 'x'], lat, {
                'standard_name': 'latitude',
                'long_name': 'latitude',
                'units': 'degrees_north',
                'axis': 'Y'
            })
            lon = (['y', 'x'], lng, {
                'standard_name': 'longitude',
                'long_name': 'longitude',
                'units': 'degrees_east',
                'axis': 'X'
            })

        coords = {
            'time': (['time'], [first_message.forecast_date], {
                'standard_name': 'time',
                'long_name': 'time',
                'units': 'seconds since 2010-01-01 00:00:00',
                'axis': 'T'
            }),
            'lat': lat,
            'lon': lon,
        }

        # Finally put it all together and create the xarray dataset
        return xr.Dataset(
            data_vars=data_vars,
            coords=coords,
            attrs={
                'meta': 'created with gribberish',
            }
        )

    def guess_can_open(self, filename_or_obj):
        try:
            _, ext = os.path.splitext(filename_or_obj)
        except TypeError:
            return False
        return ext in [".grib", ".grib2"]
