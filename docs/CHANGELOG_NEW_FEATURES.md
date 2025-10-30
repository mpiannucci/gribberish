# Changelog - New Features

## cfgrib Compatibility Update

### 🎯 Major Changes

#### 1. cfgrib-Compatible Variable Naming

Variables now use short, standard parameter names matching cfgrib conventions:

```python
# Before
'tmp_TMPhag_avgens', 'ugrd_UGRDhag_avgens', 'vgrd_VGRDisobar_avgens'

# After
'avg_2t', 'avg_u10', 'avg_v'
```

**Implementation:**
- New module: `gribberish/src/grib_naming.rs`
- Maps GRIB parameter codes to CF-compliant short names
- Handles level embedding (2m, 10m, 100m)
- Statistical process prefixes (avg_, mn, mx)

**Examples:**
- `TMP` → `t` (temperature)
- `TMP` at 2m → `2t`
- `UGRD` at 10m → `u10`
- `HGT` → `gh` (geopotential height)
- `APCP` → `tp` (total precipitation)
- `DSWRF` → `ssrd` (shortwave radiation)
- `PRMSL` → `prmsl` (pressure reduced to MSL)

#### 2. cfgrib-Compatible Coordinate Naming

Coordinates now have descriptive CF-compliant names:

```python
# Before
'hag', 'isobar', 'sfc', 'msl'

# After
'heightAboveGround', 'isobaricInhPa', 'surface', 'meanSea'
```

**Implementation:**
- Updated `python/src/dataset.rs` to use `cfgrib_coordinate_name()`
- All coordinates follow CF conventions
- Better interoperability with xarray/Pangeo tools

#### 3. Full Ensemble Member Support

Added proper `number` dimension for ensemble forecasts:

```python
# Before - ensemble members collapsed
avg_2t.shape: (1, 361, 720)
coords: ['time', 'latitude', 'longitude']

# After - each member preserved
avg_2t.shape: (201, 1, 361, 720)
coords: ['number', 'time', 'latitude', 'longitude']
number: [0, 1, 2, ..., 200]
```

**Implementation:**
- Extracts `perturbation_number` from GRIB messages
- Creates `number` coordinate with CF attributes:
  - `standard_name: "realization"`
  - `long_name: "ensemble member numerical id"`
  - `axis: "E"`
- Sorts data by ensemble member for proper ordering

**Benefits:**
- Proper dimension structure for ensemble analysis
- Calculate ensemble mean/std/min/max
- Select specific ensemble members
- Probabilistic forecasts

#### 4. Scalar Coordinates for Single-Level Variables

Coordinates now included even when variables don't have multiple levels:

```python
# Before
coords: ['time', 'latitude', 'longitude']
# heightAboveGround missing!

# After
coords: ['heightAboveGround', 'time', 'latitude', 'longitude']
heightAboveGround: [2.0, 10.0, 100.0]  # Values from all variables
```

**Implementation:**
- Collects all unique values across variables sharing same coordinate type
- Creates scalar coordinates with proper CF attributes
- Makes datasets fully CF-compliant

**Benefits:**
- Complete metadata for all variables
- Better xarray/Pangeo compatibility
- Matches cfgrib behavior

---

## Technical Details

### Files Modified

1. **`gribberish/src/grib_naming.rs`** (NEW)
   - Variable name mapping (GRIB → CF short names)
   - Coordinate name mapping (GRIB → CF names)
   - Statistical process handling
   - Level embedding logic
   - Comprehensive test suite

2. **`gribberish/src/lib.rs`**
   - Added `pub mod grib_naming`

3. **`python/src/dataset.rs`**
   - Import `grib_naming` functions
   - Use `cfgrib_variable_name()` for variable names
   - Use `cfgrib_coordinate_name()` for coordinates
   - Add ensemble dimension creation logic
   - Add scalar coordinate collection logic
   - Sort data by ensemble member number

### Backward Compatibility

**Breaking Changes:**
- Variable names have changed
- Coordinate names have changed
- Ensemble data now has `number` dimension

**Migration:**
Users can temporarily rename variables if needed:
```python
ds = ds.rename({
    'avg_2t': 'tmp_TMPhag_avgens',
    'avg_u10': 'ugrd_UGRDhag_avgens',
})
```

### Dependencies

**No new dependencies added!**
- cfgrib is NOT required
- Only implements cfgrib-*compatible* naming
- Fully standalone implementation

---

## Testing

### Tests Added

**Unit tests in `grib_naming.rs`:**
- `test_temperature_2m` - Variable naming for 2m temperature
- `test_temperature_2m_average` - Statistical prefix handling
- `test_temperature_2m_min` - Min/max prefix handling
- `test_wind_10m` - Wind component naming at 10m
- `test_geopotential_height` - Isobaric variable naming
- `test_coordinate_name` - Coordinate name mapping

All tests passing ✅

### Integration Tests

Tested with real GRIB2 ensemble file:
- ✅ 201 ensemble members properly separated
- ✅ All coordinates included (including heightAboveGround)
- ✅ Variable names match cfgrib conventions
- ✅ Coordinate names match cfgrib conventions
- ✅ Data properly sorted by ensemble member

---

## Performance

**No performance degradation:**
- Naming logic runs at parse time (one-time cost)
- No impact on data loading speed
- Lazy loading still works with xarray/dask

---

## Documentation

**New files:**
- `INSTALLATION_GUIDE.md` - Complete installation and migration guide
- `CHANGELOG_NEW_FEATURES.md` - This file

**Example config:**
- `gribberish-config.example.yaml` - Already existed, highlighted in docs

---

## Migration Checklist for Users

- [ ] Update variable name references in code
- [ ] Update coordinate name references in code
- [ ] Update ensemble data handling (add `number` dimension)
- [ ] Test with your GRIB files
- [ ] Update any scripts that parse variable names
- [ ] Review documentation for new features

---

## Future Work

Potential enhancements:
- [ ] Add more parameter mappings as needed
- [ ] Support for more statistical process types
- [ ] Additional coordinate systems
- [ ] User-configurable naming schemes (via config file)

---

## Summary

**This update makes gribberish a drop-in replacement for cfgrib with:**
- ✅ Compatible variable and coordinate naming
- ✅ Full ensemble support
- ✅ CF-compliant metadata
- ✅ Better xarray/Pangeo integration
- ✅ No external dependencies
- ✅ Cross-platform support
- ✅ High performance

**Result: Same conventions as cfgrib, better performance, no system dependencies!**
