"""Test that IFS SST (table 128 param 34) is correctly identified.

Regression test for a bug where ECMWF table 228 param 29 (instantaneous
10m wind gust) was mislabeled as SST via a legacy fallback table.
"""

import numpy as np
import pytest
from pathlib import Path

from gribberish import parse_grib_message

TEST_DATA = Path(__file__).resolve().parent.parent.parent / "test-data"


@pytest.fixture
def real_sst_data():
    path = TEST_DATA / "ifs_sst_param34_table128.grib1"
    if not path.exists():
        pytest.skip(f"Test file not found: {path}")
    return path.read_bytes()


@pytest.fixture
def wind_gust_data():
    path = TEST_DATA / "ifs_i10fg_param29_table228.grib1"
    if not path.exists():
        pytest.skip(f"Test file not found: {path}")
    return path.read_bytes()


class TestIFSParameterIdentification:

    def test_table128_param34_is_sst(self, real_sst_data):
        """ECMWF table 128 param 34 must be identified as SST."""
        msg = parse_grib_message(real_sst_data, 0)
        assert msg.metadata.var_abbrev == "sst"
        assert msg.metadata.var_name == "Sea surface temperature"
        assert msg.metadata.units == "K"

    def test_table228_param29_is_wind_gust(self, wind_gust_data):
        """ECMWF table 228 param 29 must NOT be identified as SST."""
        msg = parse_grib_message(wind_gust_data, 0)
        assert msg.metadata.var_abbrev == "i10fg"
        assert msg.metadata.var_abbrev != "sst"
        assert "wind gust" in msg.metadata.var_name.lower()

    def test_real_sst_values_in_kelvin(self, real_sst_data):
        """Real SST values should be in the 269-308 K range."""
        msg = parse_grib_message(real_sst_data, 0)
        values = msg.data()
        finite = values[np.isfinite(values)]
        assert finite.min() > 260, f"SST min {finite.min():.1f}K too low"
        assert finite.max() < 320, f"SST max {finite.max():.1f}K too high"
        assert 280 < finite.mean() < 290, f"SST mean {finite.mean():.1f}K out of range"

    def test_wind_gust_values_not_kelvin(self, wind_gust_data):
        """Wind gust values should be in m/s range, not Kelvin."""
        msg = parse_grib_message(wind_gust_data, 0)
        values = msg.data()
        finite = values[np.isfinite(values)]
        assert finite.min() >= 0, "Wind gust should be non-negative"
        assert finite.max() < 100, "Wind gust max should be < 100 m/s"
        assert finite.mean() < 20, "Wind gust mean should be < 20 m/s"

    def test_real_sst_land_fill_at_273(self, real_sst_data):
        """IFS SST fills land with 273.15K — verify this pattern."""
        msg = parse_grib_message(real_sst_data, 0)
        values = msg.data()
        near_273 = np.isclose(values, 273.15, atol=0.1)
        pct = 100 * near_273.sum() / len(values)
        # Land is ~22% of the globe
        assert 15 < pct < 35, f"Expected ~22% land fill at 273.15K, got {pct:.1f}%"
