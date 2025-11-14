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