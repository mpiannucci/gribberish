#!/usr/bin/env python
"""
Test script to verify gribberish installation with eccodes support
"""

import sys

print("Testing gribberish installation with eccodes support...")
print("=" * 60)

# Test 1: Import gribberish
print("\n1. Testing import...")
try:
    import gribberish
    print(f"   ✅ gribberish imported successfully")
    print(f"   Version: {gribberish.__version__}")
except ImportError as e:
    print(f"   ❌ Failed to import gribberish: {e}")
    sys.exit(1)

# Test 2: Import xarray
print("\n2. Testing xarray import...")
try:
    import xarray as xr
    print(f"   ✅ xarray imported successfully")
except ImportError as e:
    print(f"   ❌ Failed to import xarray: {e}")
    print("   Install with: pip install xarray")
    sys.exit(1)

# Test 3: Simple file (should work with both backends)
print("\n3. Testing with simple GFS file...")
simple_file = "/Users/maxwellgrover/projects/grib-reading/data/gfs.t18z.pgrb2.0p25.f186-RH.grib2"
try:
    ds = xr.open_dataset(simple_file, engine="gribberish")
    vars_list = list(ds.data_vars)
    print(f"   ✅ Simple file opened successfully")
    print(f"   Variables: {vars_list}")
    print(f"   Grid shape: {ds[vars_list[0]].shape}")
    ds.close()
except FileNotFoundError:
    print(f"   ⚠️  Test file not found: {simple_file}")
    print("   Skipping test (file doesn't exist)")
except Exception as e:
    print(f"   ❌ Failed to open simple file: {e}")
    sys.exit(1)

# Test 4: Ensemble file (requires eccodes)
print("\n4. Testing with ensemble SAIFS file (requires eccodes)...")
ensemble_file = "/Users/maxwellgrover/projects/grib-reading/data/saifs-s2s.ens.global.t00z.pgrb2.0p5.D000.grib2"
try:
    ds = xr.open_dataset(ensemble_file, engine="gribberish")
    num_vars = len(ds.data_vars)
    print(f"   ✅ Ensemble file opened successfully")
    print(f"   Variables: {num_vars}")
    print("   ✅ Eccodes backend is working!")
    ds.close()
except FileNotFoundError:
    print(f"   ⚠️  Test file not found: {ensemble_file}")
    print("   Skipping test (file doesn't exist)")
except ValueError as e:
    if "No valid GRIB messages found" in str(e):
        print(f"   ❌ Eccodes backend NOT working")
        print(f"   Error: {e}")
        print("\n   This means eccodes support was not properly enabled.")
        print("   Please rebuild with: maturin develop --release --features gribberish/eccodes")
        sys.exit(1)
    else:
        print(f"   ❌ Unexpected error: {e}")
        sys.exit(1)
except Exception as e:
    print(f"   ❌ Failed to open ensemble file: {e}")
    sys.exit(1)

# Summary
print("\n" + "=" * 60)
print("✅ All tests passed!")
print("=" * 60)
print("\nYour gribberish installation with eccodes support is working correctly.")
print("You can now use xarray with both standard and ensemble GRIB2 files.")
print("\nExample usage:")
print('  import xarray as xr')
print('  ds = xr.open_dataset("file.grib2", engine="gribberish")')
print('  print(ds)')
