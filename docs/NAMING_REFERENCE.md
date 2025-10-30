# Variable & Coordinate Naming Reference

Quick reference for cfgrib-compatible naming conventions in gribberish.

## Variable Names

### Temperature
| GRIB Parameter | Level | Variable Name | Description |
|---------------|-------|---------------|-------------|
| TMP | 2m | `2t` | 2-meter temperature |
| TMP | 2m avg | `avg_2t` | 2-meter temperature (average) |
| TMP | 2m min | `mn2t` | 2-meter minimum temperature |
| TMP | 2m max | `mx2t` | 2-meter maximum temperature |
| TMP | Isobaric | `t` | Temperature at pressure level |
| TMP | Isobaric avg | `avg_t` | Temperature at pressure level (average) |

### Wind Components
| GRIB Parameter | Level | Variable Name | Description |
|---------------|-------|---------------|-------------|
| UGRD | 10m | `u10` | U-component at 10m |
| VGRD | 10m | `v10` | V-component at 10m |
| WIND | 10m | `ws10` | Wind speed at 10m |
| UGRD | 100m | `u100m` | U-component at 100m |
| VGRD | 100m | `v100m` | V-component at 100m |
| WIND | 100m | `ws100m` | Wind speed at 100m |
| UGRD | Isobaric | `u` | U-component at pressure level |
| VGRD | Isobaric | `v` | V-component at pressure level |
| WIND | Isobaric | `ws` | Wind speed at pressure level |
| UGRD | Isobaric avg | `avg_u` | U-component (average) |

### Precipitation & Moisture
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| APCP | `tp` | Total precipitation |
| NCPCP | `cp` | Convective precipitation |
| ACPCP | `cp` | Convective precipitation |
| PRATE | `prate` | Precipitation rate |
| SPFH | `q` | Specific humidity |
| RH | `r` | Relative humidity |
| DPT | `d` | Dew point temperature |
| PWAT | `tcw` | Total column water (precipitable water) |

### Pressure
| GRIB Parameter | Level | Variable Name | Description |
|---------------|-------|---------------|-------------|
| PRMSL | MSL | `prmsl` | Pressure reduced to MSL |
| PRES | Surface | `sp` | Surface pressure |
| MSLET | MSL | `mslet` | MSL pressure (Eta model) |

### Geopotential
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| HGT | `gh` | Geopotential height |
| GP | `z` | Geopotential |

### Radiation
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| DSWRF | `ssrd` | Downward shortwave radiation |
| USWRF | `usrd` | Upward shortwave radiation |
| DLWRF | `strd` | Downward longwave radiation |
| ULWRF | `utrd` | Upward longwave radiation |
| NSWRF | `ssr` | Net shortwave radiation |
| NLWRF | `str` | Net longwave radiation |

### Cloud Cover
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| TCDC | `tcc` | Total cloud cover |
| LCDC | `lcc` | Low cloud cover |
| MCDC | `mcc` | Medium cloud cover |
| HCDC | `hcc` | High cloud cover |

### Snow
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| SNOD | `sde` | Snow depth |
| SNOWC | `snowc` | Snow cover |
| WEASD | `sd` | Snow water equivalent |

### Soil
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| SOILW | `swvl` | Soil moisture |
| TSOIL | `stl` | Soil temperature |

### Other
| GRIB Parameter | Variable Name | Description |
|---------------|---------------|-------------|
| VIS | `vis` | Visibility |
| THICK | `thick` | Thickness |
| CAPE | `cape` | CAPE |
| CIN | `cin` | Convective inhibition |
| VVEL | `w` | Vertical velocity |
| GUST | `gust` | Wind gust |

---

## Statistical Prefixes

| Process | Prefix | Example | Description |
|---------|--------|---------|-------------|
| Average | `avg_` | `avg_2t` | Time-averaged value |
| Minimum | `mn` or `mn_` | `mn2t` | Minimum value |
| Maximum | `mx` or `mx_` | `mx2t` | Maximum value |
| Accumulation | (implicit) | `tp` | Accumulated value |
| Standard Deviation | `std_` | `std_t` | Standard deviation |
| RMS | `rms_` | `rms_t` | Root mean square |

---

## Coordinate Names

### Vertical Coordinates
| Old Name | New Name | Description | Example Values |
|----------|----------|-------------|----------------|
| `hag` | `heightAboveGround` | Height above ground | [2.0, 10.0, 100.0] |
| `isobar` | `isobaricInhPa` | Pressure levels | [20000, 50000, 85000, 100000] |
| `sfc` | `surface` | Ground/water surface | [0.0] |
| `msl` | `meanSea` | Mean sea level | [0.0] |

### Other Coordinates
| Coordinate | Description | Example Values |
|-----------|-------------|----------------|
| `number` | Ensemble member ID | [0, 1, 2, ..., 200] |
| `time` | Forecast/analysis time | datetime array |
| `latitude` | Latitude | [-90.0, ..., 90.0] |
| `longitude` | Longitude | [0.0, ..., 359.5] |

### Full List of Surface Types
| Surface Type | Coordinate Name |
|-------------|-----------------|
| Ground or Water | `surface` |
| Cloud Base | `cloudBase` |
| Cloud Top | `cloudTop` |
| Isotherm Zero | `isothermZero` |
| Adiabatic Condensation | `adiabaticCondensation` |
| Maximum Wind | `maxWind` |
| Tropopause | `tropopause` |
| Nominal Top | `nominalTop` |
| Sea Bottom | `seaBottom` |
| Entire Atmosphere | `atmosphere` |
| Mean Sea Level | `meanSea` |
| Altitude above MSL | `altitude` |
| Height above ground | `heightAboveGround` |
| Sigma level | `sigma` |
| Hybrid level | `hybrid` |
| Depth below land | `depthBelowLand` |
| Isobaric surface | `isobaricInhPa` |
| Depth below sea | `depthBelowSea` |
| Depth below water | `depthBelowWater` |
| Level of free convection | `levelOfFreeConvection` |
| Pressure from ground | `pressureFromGround` |
| Planetary boundary layer | `planetaryBoundaryLayer` |

---

## Dimension Ordering

Standard dimension order for variables:

```python
[number, time, vertical_level, latitude, longitude]
```

**Examples:**
```python
# Ensemble 2m temperature
avg_2t.dims: ('number', 'time', 'latitude', 'longitude')
avg_2t.shape: (201, 1, 361, 720)

# Ensemble temperature at pressure levels
avg_t.dims: ('number', 'time', 'isobaricInhPa', 'latitude', 'longitude')
avg_t.shape: (201, 1, 4, 361, 720)

# Single surface variable
tp.dims: ('time', 'latitude', 'longitude')
tp.shape: (1, 361, 720)
```

---

## Quick Migration Examples

### Example 1: Basic Variables
```python
# Old code
temp = ds['tmp_TMPhag_avgens']
u_wind = ds['ugrd_UGRDhag_avgens']
precip = ds['apcp']

# New code
temp = ds['avg_2t']
u_wind = ds['avg_u10']
precip = ds['tp']
```

### Example 2: Coordinates
```python
# Old code
heights = ds.coords['hag']
pressures = ds.coords['isobar']

# New code
heights = ds.coords['heightAboveGround']
pressures = ds.coords['isobaricInhPa']
```

### Example 3: Ensemble Data
```python
# Old code - missing ensemble info
data = ds['tmp_TMPhag_avgens']  # Shape: (1, 361, 720)

# New code - full ensemble
data = ds['avg_2t']  # Shape: (201, 1, 361, 720)
members = ds.coords['number']  # [0, 1, ..., 200]

# Select member 5
member_5 = data.sel(number=5)

# Ensemble mean
ens_mean = data.mean(dim='number')
```

---

## Pattern Recognition

### Variable Name Patterns

1. **Level embedded in name**: `2t`, `u10`, `v100m`
   - Look for number + unit (m) in variable name

2. **Statistical prefix**: `avg_`, `mn`, `mx`
   - Indicates temporal processing

3. **Standard abbreviations**: Use lowercase, short forms
   - `tp` not `APCP`
   - `gh` not `HGT`
   - `prmsl` not `PRMSL`

### Coordinate Name Patterns

1. **Descriptive names**: Full words, camelCase
   - `heightAboveGround` not `hag`
   - `isobaricInhPa` not `isobar`

2. **CF-compliant**: Follow CF conventions
   - Standard names, units, axis attributes

---

## Testing Your Migration

```python
import xarray as xr

# Open file with new backend
ds = xr.open_dataset('test.grib2', engine='gribberish')

# Check variable names
print("Variables:", list(ds.data_vars))
# Should see: avg_2t, avg_u10, tp, etc.

# Check coordinates
print("Coordinates:", list(ds.coords))
# Should see: number, time, heightAboveGround, isobaricInhPa, etc.

# Check ensemble dimension
if 'number' in ds.coords:
    print(f"Ensemble members: {len(ds.coords['number'])}")

# Check shapes
for var in ds.data_vars:
    print(f"{var}: {ds[var].dims} {ds[var].shape}")
```

---

## Common Issues

### Issue: Old variable names
**Symptom:** Getting KeyError for old names like `tmp_TMPhag_avgens`

**Solution:** Update to new names:
```python
# temp = ds['tmp_TMPhag_avgens']  # ❌ Old
temp = ds['avg_2t']               # ✅ New
```

### Issue: Missing ensemble dimension
**Symptom:** Data shape is (1, 361, 720) instead of (201, 1, 361, 720)

**Solution:** Update to latest gribberish version

### Issue: Coordinate not found
**Symptom:** KeyError for 'hag' or 'isobar'

**Solution:** Use new coordinate names:
```python
# heights = ds.coords['hag']               # ❌ Old
heights = ds.coords['heightAboveGround']   # ✅ New
```

---

## Additional Resources

- **Full installation guide:** `INSTALLATION_GUIDE.md`
- **Change details:** `CHANGELOG_NEW_FEATURES.md`
- **Config examples:** `gribberish-config.example.yaml`
- **Code examples:** `python/examples/`
