# Python API Reference

The gribberish Python package provides high-level interfaces for reading GRIB2 files.

::::{tab-set}

:::{tab-item} Xarray
```python
import xarray as xr

ds = xr.open_dataset('file.grib2', engine='gribberish')
```
:::

:::{tab-item} Direct API
```python
import gribberish

messages = gribberish.parse(data)
```
:::

:::{tab-item} Zarr Codec
```python
from gribberish.zarr import GribberishCodec

zarr.codecs.register_codec(GribberishCodec)
```
:::

::::

## Core Module
```{eval-rst}
.. currentmodule:: gribberish

.. automodule:: gribberish
   :members:
   :undoc-members:
   :show-inheritance:
```

## Xarray Backend
```{eval-rst}
.. autoclass:: gribberish.GribberishBackend
   :members:
   :undoc-members:
   :show-inheritance:
```

:::{note}
The xarray backend automatically handles coordinate detection and dimension naming according to CF conventions.
:::

## Zarr Integration
```{eval-rst}
.. automodule:: gribberish.zarr
   :members:
   :undoc-members:
   :show-inheritance:
```

:::{tip}
The Zarr codec is particularly useful when working with cloud storage and kerchunk for creating virtual datasets.
:::

## Type Definitions
```{eval-rst}
.. autoclass:: gribberish.GribMessage
   :members:
   :undoc-members:
```