# gribberish-python

Python bindings to `gribberish`

## Installing 

### With `pip`

This module has not been added to `pypi` yet, so for now install with git:

```
pip install -e git+https://github.com/mpiannucci/gribberish.git#egg=gribberish\&subdirectory=python
```

### Manually

```bash
pip install -e . 
```

## Usage

This module can be used directly, or via `xarray`. 

For direct usage, see [dump_raster.py](./examples/dump_raster.py). 

To use with `xarray`, simply specify the `gribberish` backend when loading a `grib2` file: 

```
import xarray as xr
ds = xr.open_dataset('gfswave.20210826.t12z.atlocn.0p16.f000.grib2', engine='gribberish')
```

See [`xarray_usage.ipynb`](./examples/xarray_usage.ipynb) for details.