# Gribberish Installation & Migration Guide

## Table of Contents
- [Quick Start](#quick-start)
- [Installation Options](#installation-options)
- [What's New](#whats-new)
- [Migration Guide](#migration-guide)
- [Usage Examples](#usage-examples)
- [Configuration](#configuration)

---

## Quick Start

### Basic Installation

```bash
# Install from PyPI (when published)
pip install gribberish

# Or install from source
git clone https://github.com/mpiannucci/gribberish.git
cd gribberish/python
pip install -e .
```

### With Optional Dependencies

```bash
# For xarray integration
pip install gribberish[xarray]

# For Kerchunk support
pip install gribberish[kerchunk]

# For Zarr v3 support
pip install gribberish[zarr]

# Everything (for development)
pip install gribberish[dev]
```

---

## Installation Options

### 1. Standard Installation (Pure Python + Rust)

**No external dependencies required** - uses native Rust GRIB decoder.

```bash
pip install gribberish
```

✅ Works on all platforms (Linux, macOS, Windows)
✅ No system libraries needed
✅ Fast pure-Rust implementation

### 2. With eccodes Support (Optional)

For maximum compatibility with eccodes-based tools:

```bash
# Install eccodes first (system dependency)
# Ubuntu/Debian:
sudo apt-get install libeccodes-dev

# macOS:
brew install eccodes

# Then install gribberish with eccodes feature
pip install gribberish[eccodes]
```

### 3. From Source (Development)

```bash
git clone https://github.com/mpiannucci/gribberish.git
cd gribberish

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and install Python package
cd python
pip install maturin
maturin develop --release
```

---

## What's New

This version includes major improvements for **cfgrib compatibility** and **ensemble support**.

### 🎯 1. cfgrib-Compatible Variable Naming

Variables now use short, standard names that match cfgrib conventions:

**Before:**
```python
['vgrd_VGRDisobar_avgens', 'tmp_TMPhag_avgens', 'ugrd_UGRDhag_avgens',
 'wind_WINDhag_avgens', 'tmp_TMPhag_minens', 'tmp_TMPhag_maxens',
 'dswrf', 'hgt', 'apcp', 'prmsl']
```

**After:**
```python
['avg_v', 'avg_2t', 'avg_u10', 'avg_ws10', 'mn2t', 'mx2t',
 'ssrd', 'avg_gh', 'tp', 'avg_prmsl']
```

**Key improvements:**
- **Short parameter names**: `t` (temperature), `u`/`v` (wind), `gh` (geopotential height), `tp` (total precipitation)
- **Level embedding**: `2t` (2m temperature), `u10` (10m u-wind), `ws100m` (100m wind speed)
- **Statistical prefixes**: `avg_` (average), `mn` (minimum), `mx` (maximum)
- **Standard abbreviations**: `ssrd` (shortwave radiation), `prmsl` (pressure reduced MSL)

### 🎯 2. cfgrib-Compatible Coordinate Naming

Coordinates now have descriptive CF-compliant names:

**Before:**
```python
coords: ['time', 'hag', 'isobar', 'latitude', 'longitude']
```

**After:**
```python
coords: ['number', 'time', 'heightAboveGround', 'isobaricInhPa',
         'surface', 'meanSea', 'latitude', 'longitude']
```

**Key improvements:**
- `heightAboveGround` instead of `hag` (2m, 10m, 100m levels)
- `isobaricInhPa` instead of `isobar` (pressure levels)
- `surface` for ground-level variables
- `meanSea` for MSL variables
- All coordinates follow CF conventions

### 🎯 3. Ensemble Member Support

Full support for ensemble forecasts with proper `number` dimension:

**Before:**
```python
# All 201 ensemble members collapsed into single values
avg_2t.shape: (1, 361, 720)
coords: ['time', 'latitude', 'longitude']
```

**After:**
```python
# Each ensemble member preserved
avg_2t.shape: (201, 1, 361, 720)
coords: ['number', 'time', 'latitude', 'longitude']
number: [0, 1, 2, ..., 200]
```

**Key improvements:**
- `number` coordinate for ensemble member identification
- Proper dimension ordering: `[number, time, level, latitude, longitude]`
- CF-compliant attributes: `standard_name="realization"`, `axis="E"`
- Data sorted by ensemble member number

### 🎯 4. Scalar Coordinates for Single-Level Variables

Coordinates are now included even for single-level variables:

**Before:**
```python
# heightAboveGround missing if each variable has only one level
coords: ['time', 'latitude', 'longitude']
```

**After:**
```python
# All coordinate values included
coords: ['heightAboveGround', 'time', 'latitude', 'longitude']
heightAboveGround: [2.0, 10.0, 100.0]  # From different variables
```

**Key improvements:**
- All vertical coordinates included (even scalars)
- Collects unique values across all variables
- Makes datasets fully CF-compliant
- Better xarray/Pangeo compatibility

---

## Migration Guide

### For Users Upgrading from Previous Versions

#### Variable Names

If your code references specific variable names, update them:

```python
# Old code
ds = xr.open_dataset("file.grib2", engine="gribberish")
temp = ds['tmp_TMPhag_avgens']
u_wind = ds['ugrd_UGRDhag_avgens']
precip = ds['apcp']

# New code
ds = xr.open_dataset("file.grib2", engine="gribberish")
temp = ds['avg_2t']        # 2m temperature, averaged
u_wind = ds['avg_u10']     # 10m u-wind, averaged
precip = ds['tp']          # total precipitation
```

#### Coordinate Names

Update coordinate references:

```python
# Old code
height_levels = ds.coords['hag']
pressure_levels = ds.coords['isobar']

# New code
height_levels = ds.coords['heightAboveGround']
pressure_levels = ds.coords['isobaricInhPa']
```

#### Ensemble Data

Ensemble forecasts now have proper dimensions:

```python
# Old code - needed manual reshaping
data = ds['tmp_TMPhag_avgens']  # Shape: (1, 361, 720)
# Missing ensemble member information!

# New code - properly structured
data = ds['avg_2t']  # Shape: (201, 1, 361, 720)
ensemble_members = ds.coords['number']  # [0, 1, 2, ..., 200]

# Select specific ensemble member
member_5 = data.sel(number=5)

# Calculate ensemble mean
ensemble_mean = data.mean(dim='number')

# Calculate ensemble spread
ensemble_std = data.std(dim='number')
```

### Backward Compatibility

If you need the old variable names temporarily, you can rename them:

```python
ds = xr.open_dataset("file.grib2", engine="gribberish")

# Rename back to old names if needed
ds = ds.rename({
    'avg_2t': 'tmp_TMPhag_avgens',
    'avg_u10': 'ugrd_UGRDhag_avgens',
    # ... etc
})
```

---

## Usage Examples

### Basic Usage

```python
import xarray as xr

# Open GRIB2 file
ds = xr.open_dataset("data.grib2", engine="gribberish")

# View available variables
print(ds.data_vars)

# View coordinates
print(ds.coords)

# Access data
temperature = ds['avg_2t']  # 2m temperature
u_wind_10m = ds['avg_u10']  # 10m u-wind component
precip = ds['tp']           # total precipitation
```

### Working with Ensemble Data

```python
# Open ensemble forecast
ds = xr.open_dataset("ensemble.grib2", engine="gribberish")

# Check ensemble members
print(f"Ensemble members: {ds.coords['number'].values}")
# Output: [0, 1, 2, ..., 200]

# Select specific member
control = ds.sel(number=0)

# Calculate ensemble statistics
ens_mean = ds.mean(dim='number')
ens_std = ds.std(dim='number')
ens_min = ds.min(dim='number')
ens_max = ds.max(dim='number')

# Probabilistic forecast (e.g., probability of temperature > 300K)
prob_hot = (ds['avg_2t'] > 300).sum(dim='number') / len(ds.coords['number'])
```

### Working with Multiple Levels

```python
# Access data with vertical levels
ds = xr.open_dataset("multi_level.grib2", engine="gribberish")

# Check available levels
print(f"Height levels: {ds.coords['heightAboveGround'].values}")
# Output: [2.0, 10.0, 100.0]

print(f"Pressure levels: {ds.coords['isobaricInhPa'].values}")
# Output: [20000, 50000, 85000, 100000]

# Select specific level
temp_at_850 = ds['avg_t'].sel(isobaricInhPa=85000)
wind_at_10m = ds['avg_u10']  # Already at 10m
```

### Direct API (Without xarray)

```python
from gribberish import parse_grib_dataset

# Read GRIB file
with open('data.grib2', 'rb') as f:
    data = f.read()

# Parse to dictionary structure
dataset = parse_grib_dataset(data)

# Access metadata
print("Variables:", dataset['data_vars'].keys())
print("Coordinates:", dataset['coords'].keys())

# Access variable data
temp_data = dataset['data_vars']['avg_2t']
print("Dimensions:", temp_data['dims'])
print("Shape:", temp_data['values']['shape'])
print("Attributes:", temp_data['attrs'])
```

---

## Configuration

### Custom Parameter Definitions

Create a config file to add custom parameters not in WMO tables:

**gribberish-config.yaml:**
```yaml
version: "1.0"

# Add custom parameters (local use codes 192-254)
parameters:
  - discipline: 0           # Meteorological products
    category: 192          # Local use category
    number: 1
    name: "Custom Temperature"
    abbreviation: "CTEMP"
    unit: "K"
    description: "Organization-specific temperature"

  - discipline: 10          # Oceanographic products
    category: 4            # Waves
    number: 200
    name: "Custom Wave Direction"
    abbreviation: "WDIRC"
    unit: "degrees"

# Backend preferences
backend:
  preferred: "native"      # or "eccodes", "auto"
  fallback: true
```

**Usage in Python:**
```python
from gribberish.config import ConfigLoader, init_registry

# Load configuration
config = ConfigLoader.from_yaml_file('gribberish-config.yaml')

# Initialize global registry
init_registry(config)

# Now custom parameters will be recognized
ds = xr.open_dataset('custom_data.grib2', engine='gribberish')
custom_temp = ds['ctemp']  # Your custom parameter!
```

### Advanced Configuration

See `gribberish-config.example.yaml` for full configuration options including:
- Custom template definitions
- Field transformations
- Unit conversions
- Data clipping/scaling

---

## Comparison: gribberish vs cfgrib

### When to Use gribberish

✅ **No system dependencies** - Pure Python + Rust, no eccodes required
✅ **Fast** - Rust implementation with lazy loading
✅ **Cross-platform** - Works on Linux, macOS, Windows
✅ **Extensible** - Config system for custom parameters
✅ **Modern Python** - Python 3.11+ support
✅ **Multiple backends** - Native or eccodes optional

### When to Use cfgrib

- Maximum compatibility with existing ECMWF workflows
- Already have eccodes installed and working
- Need features specific to eccodes library

### Feature Comparison

| Feature | gribberish | cfgrib |
|---------|-----------|--------|
| Variable naming | ✅ cfgrib-compatible | ✅ Native |
| Coordinate naming | ✅ cfgrib-compatible | ✅ Native |
| Ensemble support | ✅ Full `number` dimension | ✅ Full support |
| System dependencies | ❌ None (pure Rust) | ⚠️ Requires eccodes |
| Custom parameters | ✅ Config file | ⚠️ Requires eccodes mod |
| Performance | ⚡ Fast (Rust) | ✅ Good |
| Platform support | ✅ Linux/Mac/Windows | ⚠️ Limited Windows |

---

## Troubleshooting

### Issue: Variables have unexpected names

**Problem:** You're seeing variable names like `vgrd_VGRDisobar_avgens` instead of `avg_v`

**Solution:** Make sure you're using the latest version of gribberish. Update with:
```bash
pip install --upgrade gribberish
```

### Issue: Missing ensemble dimension

**Problem:** Ensemble data doesn't have a `number` coordinate

**Solution:** This is fixed in the latest version. Update gribberish:
```bash
pip install --upgrade gribberish
```

### Issue: heightAboveGround coordinate missing

**Problem:** Dataset doesn't include `heightAboveGround` coordinate

**Solution:** This is fixed in the latest version. Scalar coordinates are now included:
```bash
pip install --upgrade gribberish
```

### Issue: Custom parameters not recognized

**Problem:** Your custom parameters aren't showing up

**Solution:** Make sure you initialize the registry before opening GRIB files:
```python
from gribberish.config import ConfigLoader, init_registry

config = ConfigLoader.from_yaml_file('gribberish-config.yaml')
init_registry(config)  # Call this BEFORE opening files

ds = xr.open_dataset('data.grib2', engine='gribberish')
```

---

## Performance Tips

### 1. Use Lazy Loading

gribberish supports lazy loading through xarray:

```python
import dask

# Open with dask for lazy loading
ds = xr.open_dataset('large_file.grib2', engine='gribberish', chunks='auto')

# Data is only loaded when accessed
subset = ds['avg_2t'].sel(latitude=slice(30, 50))
result = subset.mean().compute()  # Load only what's needed
```

### 2. Filter Variables Early

```python
# Only load specific variables
ds = xr.open_dataset(
    'data.grib2',
    engine='gribberish',
    backend_kwargs={'only_variables': ['avg_2t', 'avg_u10', 'tp']}
)
```

### 3. Use Kerchunk for Cloud Data

```python
from gribberish.kerchunk import generate_reference

# Generate reference for cloud-optimized access
reference = generate_reference('s3://bucket/data.grib2')

# Open with fsspec
import fsspec
import xarray as xr

mapper = fsspec.get_mapper('reference://', fo=reference)
ds = xr.open_zarr(mapper)
```

---

## Getting Help

- **Documentation:** [GitHub Repository](https://github.com/mpiannucci/gribberish)
- **Issues:** [GitHub Issues](https://github.com/mpiannucci/gribberish/issues)
- **Examples:** See `python/examples/` directory
- **Config Examples:** See `gribberish-config.example.yaml`

---

## Summary of Key Changes

### ✅ What's New
1. **cfgrib-compatible variable naming** (`avg_2t`, `u10`, `gh`, `tp`, etc.)
2. **cfgrib-compatible coordinate naming** (`heightAboveGround`, `isobaricInhPa`, etc.)
3. **Full ensemble support** with `number` dimension
4. **Scalar coordinates included** for all vertical levels
5. **No cfgrib dependency required** - fully independent implementation

### ✅ What Stays the Same
- Config file system for custom parameters
- Backend flexibility (native or eccodes)
- xarray integration
- Kerchunk support
- Cross-platform compatibility
- High performance

### 🎉 Result
**Drop-in replacement for cfgrib with better performance and no system dependencies!**
