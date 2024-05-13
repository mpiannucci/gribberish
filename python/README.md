# gribberishpy

Python bindings to `gribberish`

## Installing 

### With `pip`

This module has not been added to `pypi` yet, so for now install with git:

```bash
pip install -e "gribberish@git+https://github.com/mpiannucci/gribberish.git#egg=gribberish\&subdirectory=python"
```

With optional `xarray` support: 

```
pip install -e "gribberish[xarray]@git+https://github.com/mpiannucci/gribberish.git#egg=gribberish\&subdirectory=python"
```

With optional `kerchunk` support: 

```bash
pip install -e "gribberish[kerchunk]@git+https://github.com/mpiannucci/gribberish.git#egg=gribberish\&subdirectory=python"
```

### Manually

```bash
pip install -e . 
```

## Usage

This module can be used directly, or via `xarray`. 

For direct usage, see [dump_raster.py](./examples/dump_raster.py). 

### `xarray`

To use with `xarray`, simply specify the `gribberish` backend when loading a `grib2` file: 

```
import xarray as xr
ds = xr.open_dataset('gfswave.20210826.t12z.atlocn.0p16.f000.grib2', engine='gribberish')
```

Some examples are provided: 

- [`xarray_usage.ipynb`](./examples/xarray_usage.ipynb) shows how to load a single GFS Wave model grib2 file
- [`hrrr.ipynb`](./examples/hrrr.ipynb) shows how to load a single HRRR model grib2 file. There are multiple time and vertical coordinates in this file so it shows how the full dataset can be loaded or filtered down to only what is desired. It also demonstrates how to select data at a given point in space using the non regular gridded coordinate system.
- [`nwps.ipynb`](./examples/nwps.ipynb) shows how to load an entire NWPS model output that is distributed in a single grib2 file.
- [`gfs.ipynb`](./examples/gfs.ipynb) shows how to load a single GFS grib2 output file.
- [`read_radar.ipynb`](./examples/read_radar.ipynb) shows how to load a single radar file from a single uncompressed [`MRMS`](https://www.nssl.noaa.gov/projects/mrms/) grib2 file.

### `kerchunk`

This package also supports building virtual datasets with [`kerchunk`](https://github.com/fsspec/kerchunk). Two examples are provided: 

- [`kerchunk_gefs_wave.ipynb`](./examples/kerchunk_gefs_wave.ipynb) shows how to build a single virtual dataset from an entire GEFS Wave Ensemble model run (30 ensemble members, 384 hour time horizon)
- [`kerchunk_hrrr_subhourly.ipynb`](./examples/kerchunk_hrrr_subhourly.ipynb) shows how to build a single virtual dataset from an entire HRRR subhourly surface model run. This results in a virtual dataset with data at 15 minute time intervals over the following 18 hours.