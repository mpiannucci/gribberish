# Quick Start

This guide will help you get started with gribberish.

## Basic Usage

### Reading a GRIB2 File

The simplest way to read a GRIB2 file is using xarray:
```python
import xarray as xr

# Open a GRIB2 file
ds = xr.open_dataset('path/to/file.grib2', engine='gribberish')

# Display the dataset
print(ds)
```

### Direct API Usage

For more control, you can use the gribberish API directly:
```python
import gribberish

# Open and read a GRIB2 file
with open('path/to/file.grib2', 'rb') as f:
    data = f.read()
    
# Parse the GRIB2 data
messages = gribberish.parse(data)

# Access individual messages
for msg in messages:
    print(f"Parameter: {msg.parameter}")
    print(f"Level: {msg.level}")
    print(f"Data shape: {msg.data.shape}")
```

## Working with Multiple Files

### Using Xarray with Dask
```python
import xarray as xr

# Open multiple files lazily
files = ['file1.grib2', 'file2.grib2', 'file3.grib2']
datasets = [xr.open_dataset(f, engine='gribberish', chunks='auto') 
            for f in files]

# Combine along time dimension
combined = xr.concat(datasets, dim='time')
```

## Zarr Integration

gribberish provides a custom Zarr codec for reading GRIB2 data:
```python
from gribberish.zarr import GribberishCodec
import zarr

# Register the codec
zarr.codecs.register_codec(GribberishCodec)

# Use with zarr arrays
# (typically used with VirtualiZarr or kerchunk)
```

## Dimension Naming

When variables in the same group span different sets of values along a shared
coordinate type (vertical levels, valid times, ensemble members, percentiles,
probability thresholds), each distinct value set becomes its own dimension
with an index suffix: `isobar_0`, `isobar_1`, `time_1`, `number_1`, and so on.

The suffix depends only on the collection of value sets in the file:
re-reading a file, or reading another file with the same schema, produces
identical names, so same-schema datasets can be joined. Names are **not
stable across schema changes** — if a model upgrade adds a field with a new
level set, suffixes can shift to different value sets even for variables that
did not change. Avoid hardcoding suffixed names; instead, discover the
dimension from the variable and select by coordinate value:
```python
level_dim = next(d for d in ds["tmp"].dims if d.startswith("isobar"))
ds["tmp"].sel({level_dim: 50000})  # 500 hPa
```

Filtering to variables that share a single value set (e.g. with
`only_variables`) removes the conflict, and the dimension takes the plain,
stable name (`isobar`).

## Performance Tips

1. **Use chunks** - When working with large files, use dask chunks:
```python
   ds = xr.open_dataset('large.grib2', engine='gribberish', chunks={'time': 1})
```

2. **Filter messages** - Read only what you need:
```python
   ds = xr.open_dataset('file.grib2', engine='gribberish',
                        only_variables=["air_temperature"])
```

## Next Steps

* Check out the [examples](examples/index.md) for more detailed use cases
* Read the [API documentation](api/python.md) for complete reference
* See the [notebooks](https://github.com/mpiannucci/gribberish/tree/main/examples) in the repository