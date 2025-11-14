# gribberish

**Read GRIB2 files with Rust and Python**

::::{grid} 1 2 2 3
:gutter: 2

:::{grid-item-card} 🚀 Fast
:text-align: center
Rust-powered core for blazing fast GRIB2 parsing
:::

:::{grid-item-card} 🐍 Pythonic
:text-align: center
Native Python bindings with NumPy and Xarray integration
:::

:::{grid-item-card} ☁️ Cloud-Ready
:text-align: center
Zarr codec support for cloud-native workflows
:::
::::

## Quick Start

:::::{tab-set}

::::{tab-item} pip
```bash
pip install gribberish
```
::::

::::{tab-item} conda
```bash
conda install -c conda-forge gribberish  # Coming soon
```
::::

::::{tab-item} From source
```bash
git clone https://github.com/mpiannucci/gribberish
cd gribberish/python
pip install maturin
maturin develop --release
```
::::

:::::

## Example Usage
```python
import xarray as xr

# Load GRIB2 file using gribberish backend
ds = xr.open_dataset('data.grib2', engine='gribberish')
print(ds)
```

## Documentation

::::{grid} 1 2 2 2
:gutter: 3

:::{grid-item-card}
:link: quickstart
:link-type: doc

**Getting Started** 🎯
^^^
New to gribberish? Start here for installation and basic usage.
:::

:::{grid-item-card}
:link: api/python
:link-type: doc

**API Reference** 📖
^^^
Complete API documentation for Python and Rust interfaces.
:::

:::{grid-item-card}
:link: examples/index
:link-type: doc

**Examples** 💡
^^^
Practical examples and Jupyter notebooks for common use cases.
:::

:::{grid-item-card}
:link: contributing
:link-type: doc

**Contributing** 🤝
^^^
Learn how to contribute to gribberish development.
:::

::::
```{toctree}
:hidden:
:maxdepth: 2

quickstart
installation
```
```{toctree}
:hidden:
:caption: API Reference
:maxdepth: 2

api/python
api/rust
```
```{toctree}
:hidden:
:caption: Examples
:maxdepth: 2

examples/index
```
```{toctree}
:hidden:
:caption: Development
:maxdepth: 1

contributing
changelog
GitHub <https://github.com/mpiannucci/gribberish>
```

## Features

- **High Performance**: Rust implementation ensures fast GRIB2 parsing
- **Xarray Backend**: Seamlessly integrate with the PyData ecosystem
- **Memory Efficient**: Stream large files without loading everything into memory
- **Cloud Native**: Zarr codec for working with cloud storage
- **Type Safe**: Full type hints for better IDE support

## License

MIT License - see [LICENSE](https://github.com/mpiannucci/gribberish/blob/main/LICENSE) for details.