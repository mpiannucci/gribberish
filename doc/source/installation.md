# Installation

## Requirements

* Python 3.9 or higher
* Rust 1.70 or higher (only for building from source)

## Install from PyPI

The easiest way to install gribberish is from PyPI:
```bash
pip install gribberish
```

### Optional Dependencies

For xarray support:
```bash
pip install "gribberish[xarray]"
```

For kerchunk support:
```bash
pip install "gribberish[kerchunk]"
```

For zarr support:
```bash
pip install "gribberish[zarr]"
```

Install all optional dependencies:
```bash
pip install "gribberish[xarray,kerchunk,zarr]"
```

## Install from Source

### Prerequisites

First, install Rust if you haven't already:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone and Install
```bash
# Clone the repository
git clone https://github.com/mpiannucci/gribberish.git
cd gribberish

# Install maturin (Rust-Python build tool)
pip install maturin

# Build and install the Python package
cd python
maturin develop --release
```

### Development Installation

For development with live reloading:
```bash
cd python
pip install -e .
```

## Verify Installation
```python
import gribberish
print(gribberish.__version__)
```