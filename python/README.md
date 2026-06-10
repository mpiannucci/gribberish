# gribberish python

Read [GRIB 2](https://en.wikipedia.org/wiki/GRIB) files with Python. This package is a Python wrapper around the Rust library [gribberish](https://crates.io/crates/gribberish).

There is support for `xarray` and [`VirtualiZarr`](https://virtualizarr.readthedocs.io/) integration, as well as just reading the data directly into `numpy` arrays.

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

With optional `zarr` support:

```bash
pip install "gribberish[zarr]"
```

With optional [`VirtualiZarr`](https://virtualizarr.readthedocs.io/) support:

```bash
pip install "gribberish[virtualizarr]"
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

#### Dimension naming when variables disagree

Within a group, variables that share a coordinate type (vertical level, valid
time, ensemble member, percentile, probability threshold) but span *different
value sets* each get their own dimension, disambiguated with an index suffix:
`isobar_0`, `isobar_1`, `time_1`, `number_1`, …. For example, GEFS `pgrb2a`
files carry isobaric variables on 10-, 11- and 12-level sets, which surface as
three coordinates `isobar_0`/`isobar_1`/`isobar_2`.

The suffix is **deterministic but positional**. Names depend only on the
collection of value sets present in the file, so re-reading a file — or
reading another file with the same schema, even with its messages stored in a
different order — always produces identical names, and datasets from
same-schema files can be joined (e.g. `xr.concat` across forecast hours).
Names are *not* stable across schema changes, however: if a model upgrade
adds a field with a new set of pressure levels, or changes the levels of an
existing field, suffixes can shift to different value sets, including for
variables that did not change.

To stay robust against schema changes:

- Never hardcode suffixed dimension names. Discover the dimension from the
  variable and select by coordinate *value*:

  ```python
  level_dim = next(d for d in ds["tmp"].dims if d.startswith("isobar"))
  ds["tmp"].sel({level_dim: 50000})  # 500 hPa
  ```

- Filter down to variables that share one value set — when a single set
  remains, the dimension takes the plain, stable name (no suffix):

  ```python
  ds = xr.open_dataset(path, engine="gribberish",
                       only_variables=["tmp"], group="isobar")
  ds.tmp.sel(isobar=50000)
  ```

- For pipelines that persist a schema (zarr templates, validation, archival),
  key on the coordinate values attached to each variable rather than on
  dimension names, or process variables one at a time via `only_variables`.

Some examples are provided:

- [`xarray_usage.ipynb`](./examples/xarray_usage.ipynb) shows how to load a single GFS Wave model grib2 file
- [`hrrr.ipynb`](./examples/hrrr.ipynb) shows how to load a single HRRR model grib2 file. There are multiple time and vertical coordinates in this file so it shows how the full dataset can be loaded or filtered down to only what is desired. It also demonstrates how to select data at a given point in space using the non regular gridded coordinate system.
- [`nwps.ipynb`](./examples/nwps.ipynb) shows how to load an entire NWPS model output that is distributed in a single grib2 file.
- [`gfs.ipynb`](./examples/gfs.ipynb) shows how to load a single GFS grib2 output file.
- [`read_radar.ipynb`](./examples/read_radar.ipynb) shows how to load a single radar file from a single uncompressed [`MRMS`](https://www.nssl.noaa.gov/projects/mrms/) grib2 file.

### `VirtualiZarr`

This package can build virtual datasets with [`VirtualiZarr`](https://virtualizarr.readthedocs.io/) using the `GribberishParser`. Conflicting hypercubes (e.g. the same variable at different level types, or instantaneous vs. accumulated fields) are split into nested groups, matching the way `cfgrib` breaks a file into multiple datasets:

```python
import virtualizarr as vz
from obstore.store import LocalStore
from obspec_utils.registry import ObjectStoreRegistry
from gribberish.virtualizarr import GribberishParser

registry = ObjectStoreRegistry({"file://": LocalStore()})
store = GribberishParser()("file:///path/to/file.grib2", registry)

# A single, conflict-free file opens directly:
vds = store.to_virtual_dataset()

# A file with conflicting hypercubes opens as a tree of groups:
vdt = store.to_virtual_datatree()
```

### `zarr`

This package also supports use with `zarr` for reading unmodified GRIB2 messages (arrays) as chunks using the `gribberish.zarr.GribberishCodec` codec. This is what decodes chunks at read time and is usually used indirectly via [`VirtualiZarr`](https://virtualizarr.readthedocs.io/); the codec must be importable in the environment that reads the data.