# gribberish python

Read [GRIB 2](https://en.wikipedia.org/wiki/GRIB) files with Python. This package is a Python wrapper around the Rust library [gribberish](https://crates.io/crates/gribberish).

There is support for `xarray` and `kerchunk` integration, as well as just reading the data directly into `numpy` arrays.

## Installing

### With `pip`

```bash
pip install gribberish
```

or install with git:

```bash
pip install -e "gribberish@git+https://github.com/mpiannucci/gribberish.git#egg=gribberish\&subdirectory=python"
```

With optional `xarray` support:

```
pip install "gribberish[xarray]"
```

With optional `kerchunk` support:

```bash
pip install "gribberish[kerchunk]"
```

With optional `zarr` support:

```bash
pip install "gribberish[zarr]"
```

### Manually

With pip:

```bash
pip install -e .
```

or with [maturin](https://github.com/PyO3/maturin)

```bash
pip install maturin
maturin develop
```

## Usage

This module can be used directly, or via `xarray`.

For direct usage, see [dump_dataset.py](./examples/dump_dataset.py) or compare with `eccodes` usage and performance in [bench.py](./examples/bench.py).

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

### `zarr`

This package also supports use with `zarr` for reading unmodified GRIB2 messages (arrays) as chunks using the `gribberish.zarr.GribberishCodec` codec. This usually will not be used directly, but with [`VirtualiZarr`](https://virtualizarr.readthedocs.io/en/latest/) or [`kerchunk`](https://github.com/fsspec/kerchunk)

Examples to come soon.