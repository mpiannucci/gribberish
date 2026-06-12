from pathlib import Path

import pytest

xr = pytest.importorskip("xarray")

from gribberish import parse_grib_index

GRIB = Path(__file__).resolve().parents[2] / "test-data" / "gfswave.t18z.atlocn.0p16.f001.grib2"
IDX = GRIB.with_name(GRIB.name + ".idx")


def test_use_index_matches_full_scan():
    """An index file locates every message, and opening through it (index +
    per-message range reads only) yields the identical dataset to downloading
    and scanning the whole file."""
    entries = parse_grib_index(IDX.read_text(), file_size=GRIB.stat().st_size)
    assert len(entries) == 19
    assert (entries[0].var, entries[0].offset, entries[0].length) == ("WIND", 0, 41723)
    assert entries[0].level == "surface"
    assert entries[0].forecast_time == "1 hour fcst"
    # Offsets and inferred lengths tile the file exactly.
    assert entries[-1].offset + entries[-1].length == GRIB.stat().st_size

    kwargs = dict(engine="gribberish", only_variables=["wind", "htsgw"])
    full = xr.open_dataset(str(GRIB), **kwargs)
    via_index = xr.open_dataset(str(GRIB), use_index=True, **kwargs)
    xr.testing.assert_identical(full, via_index)

    # Without a sidecar: use_index=True demands one, "auto" falls back.
    no_idx = GRIB.with_name("multi_1.at_10m.t12z.f147.grib2")
    with pytest.raises(FileNotFoundError, match="No index file"):
        xr.open_dataset(str(no_idx), engine="gribberish", use_index=True)
    xr.open_dataset(str(no_idx), engine="gribberish", use_index="auto")
