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

To use with `xarray`, simply specify the `gribberish` backend when loading a `grib2` file. By default a file is split into nested groups by surface type and product kind, so open one group at a time (or the whole tree with `xr.open_datatree`):

```
import xarray as xr
ds = xr.open_dataset('gfswave.20210826.t12z.atlocn.0p16.f000.grib2', engine='gribberish', group='sfc/instant')

# Or fold everything into one dataset where possible:
ds = xr.open_dataset('gfswave.20210826.t12z.atlocn.0p16.f000.grib2', engine='gribberish', collapse_groups=True)
```

Remote files (`s3://`, `gs://`, `https://`) work the same way and are fetched
via [`obstore`](https://developmentseed.org/obstore/); pass credentials or
region through `storage_options`. For files with a sidecar index, add
`use_index=` to skip downloading the parts you don't need — see
[Sidecar index files](#sidecar-index-files-idx--index) below.

Some examples are provided:

- [`xarray_usage.ipynb`](./examples/xarray_usage.ipynb) shows how to load a single GFS Wave model grib2 file
- [`hrrr.ipynb`](./examples/hrrr.ipynb) shows how to load a single HRRR model grib2 file. There are multiple time and vertical coordinates in this file so it shows how the full dataset can be loaded or filtered down to only what is desired. It also demonstrates how to select data at a given point in space using the non regular gridded coordinate system.
- [`nwps.ipynb`](./examples/nwps.ipynb) shows how to load an entire NWPS model output that is distributed in a single grib2 file.
- [`gfs.ipynb`](./examples/gfs.ipynb) shows how to load a single GFS grib2 output file.
- [`read_radar.ipynb`](./examples/read_radar.ipynb) shows how to load a single radar file from a single uncompressed [`MRMS`](https://www.nssl.noaa.gov/projects/mrms/) grib2 file.

### `VirtualiZarr`

This package can build virtual datasets with [`VirtualiZarr`](https://virtualizarr.readthedocs.io/) using the `GribberishParser`. By default each variable is nested under its surface type and product kind (e.g. `/hag/instant`, `/sfc/instant`), matching the way `cfgrib` breaks a file into multiple datasets. This layout is **content-independent** — a variable lands at the same group path in every same-schema file, so multi-file datacubes concatenate cleanly across a forecast sequence:

```python
import virtualizarr as vz
from obstore.store import LocalStore
from obspec_utils.registry import ObjectStoreRegistry
from gribberish.virtualizarr import GribberishParser

registry = ObjectStoreRegistry({"file://": LocalStore()})
store = GribberishParser()("file:///path/to/file.grib2", registry)

# The file opens as a tree of standalone group datasets:
vdt = store.to_virtual_datatree()

# Pass collapse_groups=True to fold a conflict-free file into one root dataset
# (levels/kinds become dimensions). Cleaner per file, but the group layout then
# depends on the file's content, so the same variable can land at different
# paths across files — which breaks concatenation.
store = GribberishParser(collapse_groups=True)("file:///path/to/file.grib2", registry)
vds = store.to_virtual_dataset()
```

`GribberishParser` also accepts `use_index=` to build manifests from a sidecar
index without downloading the GRIB file — see the next section.

### Sidecar index files (`.idx` / `.index`)

Most NOAA models (GFS, HRRR, GEFS, ...) publish a wgrib2-style `.idx` text
file next to every GRIB2 file, and ECMWF open data publishes a JSON-lines
`.index` file. Both list each message's byte offset, so individual messages
can be fetched from object storage with range requests instead of downloading
the whole file. gribberish supports both formats everywhere through a single
keyword:

| `use_index=` | behavior |
|---|---|
| `False` (default) | download the whole file and scan it |
| `"auto"` | probe the known index names; silently fall back to a full read if none exists |
| a suffix (`".idx"`, `".index"`, `".inv"`, ...) | probe only that name; raise `FileNotFoundError` if missing |

**With `xarray`**, the index is combined with the variable filters so only the
messages you keep are downloaded (the full filters still run on the fetched
messages, so results are identical to a full scan):

```python
ds = xr.open_dataset(
    "s3://noaa-hrrr-bdp-pds/hrrr.20260609/conus/hrrr.t01z.wrfsfcf01.grib2",
    engine="gribberish",
    use_index="auto",
    only_variables=["tmp", "gust"],
    group="sfc/instant",
    storage_options={"region": "us-east-1", "skip_signature": True},
)
```

**With `VirtualiZarr`**, the index locates every message and only each
message's leading header bytes are fetched (concurrently) to read its
metadata — the data sections are never downloaded, since manifest chunks
point back at the original file. Virtualizing a 550 MB GFS file this way
transfers ~3 MB and takes about a second, and produces byte-identical
manifests to a full scan:

```python
parser = GribberishParser(use_index="auto")
```

**Low level**, `parse_grib_index` parses either format into entries with
`offset`, `length`, `var`, `level`, `forecast_time`, a `reference_date`, and
(for ECMWF) the raw MARS `keys` — useful for driving your own range requests:

```python
from gribberish import parse_grib_index

entries = parse_grib_index(idx_text, file_size=grib_size)
tmp_2m = [e for e in entries if e.var == "TMP" and e.level == "2 m above ground"]
```

Notes:

- The byte savings in the `xarray` path come from `only_variables` /
  `drop_variables`; with no filters every message is still fetched.
  Attribute filters (`filter_by_attrs`, ...) are not matched against the
  index — they are applied after fetching, so they are always exact.
- Index variable names occasionally differ from gribberish's (e.g. NOAA
  writes `CLMR` where gribberish uses `CLWMR`). If a filtered variable is
  unexpectedly missing, open without filters or without `use_index` to
  compare.
- cfgrib also writes files ending in `.idx` next to GRIB files — those are
  pickled cfgrib caches, unrelated to NOAA inventories, and not supported.

### `zarr`

This package also supports use with `zarr` for reading unmodified GRIB2 messages (arrays) as chunks using the `gribberish.zarr.GribberishCodec` codec. This is what decodes chunks at read time and is usually used indirectly via [`VirtualiZarr`](https://virtualizarr.readthedocs.io/); the codec must be importable in the environment that reads the data.