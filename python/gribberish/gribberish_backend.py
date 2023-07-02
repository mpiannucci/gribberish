import os
import numpy as np
import xarray as xr

from .gribberishpy import parse_grib_mapping, parse_grib_message
from xarray.backends import BackendEntrypoint


def read_binary_data(filename: str):
    # TODO: Make this streamed for large files, etc etc
    with open(filename, 'rb') as f:
        return f.read()


def extract_variable_data(grib_message):
    data = grib_message.data().reshape(grib_message.metadata.grid_shape)
    data = np.expand_dims(data, axis=0)
    crs = grib_message.metadata.crs
    standard_name = grib_message.metadata.var_name
    units = grib_message.metadata.units
    level_type = grib_message.metadata.level_type
    dims = grib_message.metadata.dims

    return (
        dims,
        data,
        {
            'standard_name': standard_name,
            'long_name': standard_name,
            'units': units,
            'crs': crs,
            'level_type': level_type
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
        var_mapping = parse_grib_mapping(raw_data, drop_variables=drop_variables)

        # For now, any unsupported products get dropped 
        keys = list(var_mapping.keys())
        for var in keys: 
            if var.startswith('(') or var == 'unknown': 
                var_mapping.pop(var, None)

        # Get all dimensions
        extra_dims = {}
        for lookup in var_mapping.values():
            dims = lookup[2].non_spatial_dims
            if 'time' in dims:
                if 'time' not in extra_dims:
                    extra_dims['time'] = set([lookup[2].forecast_date])
                else:
                    extra_dims['time'].add(lookup[2].forecast_date)
            
            if len(dims) > 1:
                level_value = lookup[2].level_value
                if dims[1] == 'seq':
                    level_value = int(level_value)

                if dims[1] not in extra_dims:
                    extra_dims[dims[1]] = set([level_value])
                else:
                    extra_dims[dims[1]].add(level_value)
        
        # Convert sets to lists
        for dim, values in extra_dims.items():
            extra_dims[dim] = list(values)

        var_lookups = {}
        for var, lookup in var_mapping.items():
            non_dims_key = lookup[2].non_dims_key
            if non_dims_key not in var_lookups:
                var_lookups[non_dims_key] = [lookup]
            else:
                var_lookups[non_dims_key].append(lookup)

        data_vars = {}
        for non_dims_key, lookups in var_lookups.items():
            # TODO: Dont assume everything has a time dimension?
            if len(lookups) == 1:
                data_vars[lookups[0][2].var_abbrev.lower()] = extract_variable_data(
                    parse_grib_message(raw_data, lookups[0][1])
                )
            else: 
                data = []
                for i in range(0, len(lookups)):
                    lookup = lookups[i]
                    data.append((i, extract_variable_data(
                        parse_grib_message(raw_data, lookup[1])
                    )[1]))

                # If there is only one non-spatial dimension, concatenate the data
                # Otherwise we have to figure out what axis to concatenate on
                if len(lookups[0][2].non_spatial_dims) == 1:
                    data_vars[lookups[0][2].var_abbrev.lower()] = (
                        lookups[0][2].dims, 
                        np.concatenate([d[1] for d in data], axis=0), 
                        {
                            'standard_name': lookups[0][2].var_name,
                            'long_name': lookups[0][2].var_name,
                            'units': lookups[0][2].units,
                            'crs': lookups[0][2].crs,
                            'level_type': lookups[0][2].level_type
                        }
                    )
                else:
                    # First we have to sort the data by the non-spatial dimensions
                    data = sorted(data, key=lambda x: (lookups[x[0]][2].forecast_date, float(lookups[x[0]][2].level_value)))
                    data = [d[1] for d in data]

                    # then reshape according to the non-spatial dimensions
                    non_spatial_dims = lookups[0][2].non_spatial_dims
                    non_spatial_dim_shape = [len(extra_dims[dim]) for dim in non_spatial_dims]
                    data = np.concatenate(data, axis=0)
                    data = data.reshape(non_spatial_dim_shape + list(data.shape[1:]))
                    data_vars[lookups[0][2].var_abbrev.lower()] = (
                        lookups[0][2].dims, 
                        data, 
                        {
                            'standard_name': lookups[0][2].var_name,
                            'long_name': lookups[0][2].var_name,
                            'units': lookups[0][2].units,
                            'crs': lookups[0][2].crs,
                            'level_type': lookups[0][2].level_type
                        }
                    )

        # Extract each variables metadata
        # data_vars = {var: extract_variable_data(parse_grib_message(
        #     raw_data, lookup[1])) for (var, lookup) in var_mapping.items()}

        # Get the coordinate arrays
        # TODO: This can be optimized
        first_message = parse_grib_message(
            raw_data,
            list(var_mapping.values())[0][1]
        )

        lat = first_message.metadata.latitudes().reshape(first_message.metadata.grid_shape)
        lng = first_message.metadata.longitudes().reshape(first_message.metadata.grid_shape)

        if first_message.metadata.is_regular_grid:
            lat = (['latitude'], lat[:, 0], {
                'standard_name': 'latitude',
                'long_name': 'latitude',
                'units': 'degrees_north',
                'axis': 'Y'
            })
            lon = (['longitude'], lng[0, :], {
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
            'latitude': lat,
            'longitude': lon,
        }

        for dim, values in extra_dims.items():
            if dim == 'time':
                coords['time'] = (['time'], extra_dims['time'], {
                    'standard_name': 'time',
                    'long_name': 'time',
                    'units': 'seconds since 2010-01-01 00:00:00',
                    'axis': 'T'
                })
            coords[dim] = (dim, values, {
                # TODO: Actual level names
                'standard_name': dim,
            })

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
