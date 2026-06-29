from pathlib import Path

import pytest

import numpy as np

zarr = pytest.importorskip("zarr")

TEST_DATA = Path(__file__).resolve().parents[2] / "test-data"

# GEFS 0.5° — first message (HGT) is a 361×720 global grid, lon 0..359.5. The
# antimeridian (180°) sits on column 360, so the wrap rolls by exactly 360.
GEAVG = "geavg.t12z.pgrb2a.0p50.f000"
GEAVG_SHAPE = (361, 720)
GEAVG_ROLL = 360


async def _decode(codec, raw, shape, dtype="float64"):
    from zarr.core.array_spec import ArraySpec, ArrayConfig
    from zarr.core.buffer import default_buffer_prototype
    from zarr.dtype import parse_dtype
    from zarr.buffer.cpu import buffer_prototype

    buffer = default_buffer_prototype().buffer.from_bytes(raw)
    decoded = await codec._decode_single(
        buffer,
        ArraySpec(
            shape=shape,
            dtype=parse_dtype(dtype, zarr_format=3),
            fill_value=0,
            prototype=buffer_prototype,
            config=ArrayConfig(order="C", write_empty_chunks=False),
        ),
    )
    return decoded.as_ndarray_like()


async def test_adjust_longitude_range_rolls_data():
    """Opt-in wrap rolls decoded data left along longitude by the split column."""
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / GEAVG).read_bytes()
    # `var` is only special-cased for 'latitude'/'longitude'; any other name
    # decodes the data array, so the name here is just a label.
    native = await _decode(GribberishCodec("HGT"), raw, GEAVG_SHAPE)
    adjusted = await _decode(
        GribberishCodec("HGT", adjust_longitude_range=True), raw, GEAVG_SHAPE
    )

    nx = GEAVG_SHAPE[1]
    cols = (np.arange(nx) + GEAVG_ROLL) % nx
    np.testing.assert_array_equal(adjusted, native[:, cols])
    assert not np.array_equal(adjusted, native)  # data really moved


async def test_adjust_longitude_range_wraps_longitude_coordinate():
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / GEAVG).read_bytes()
    native = await _decode(GribberishCodec("longitude"), raw, (GEAVG_SHAPE[1],))
    wrapped = await _decode(
        GribberishCodec("longitude", adjust_longitude_range=True), raw, (GEAVG_SHAPE[1],)
    )

    # default is unchanged 0..360
    assert native[0] == 0.0 and native[-1] == 359.5
    # wrapped is strictly monotonic over [-180, 180)
    assert wrapped[0] == -180.0 and wrapped[-1] == 179.5
    assert np.all(np.diff(wrapped) > 0)
    assert np.all((wrapped >= -180.0) & (wrapped < 180.0))


async def test_adjust_longitude_range_leaves_latitude_untouched():
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / GEAVG).read_bytes()
    native = await _decode(GribberishCodec("latitude"), raw, (GEAVG_SHAPE[0],))
    adjusted = await _decode(
        GribberishCodec("latitude", adjust_longitude_range=True), raw, (GEAVG_SHAPE[0],)
    )
    np.testing.assert_array_equal(native, adjusted)


async def test_adjust_longitude_range_noop_for_non_global_grid():
    """A regional/projected (HRRR Lambert) grid is left untouched."""
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / "hrrr.t06z.wrfsfcf01-UGRD.grib2").read_bytes()
    shape = (1059, 1799)
    native = await _decode(GribberishCodec("UGRD"), raw, shape)
    adjusted = await _decode(
        GribberishCodec("UGRD", adjust_longitude_range=True), raw, shape
    )
    np.testing.assert_array_equal(native, adjusted)


def test_codec_config_roundtrips_adjust_flag():
    from gribberish.zarr.codec import GribberishCodec

    codec = GribberishCodec("HGT", adjust_longitude_range=True)
    assert codec.to_dict() == {
        "name": "gribberish",
        "configuration": {"var": "HGT", "adjust_longitude_range": True},
    }
    assert GribberishCodec.from_dict(codec.to_dict()) == codec

    # default off omits the flag, so existing stored metadata round-trips unchanged
    plain = GribberishCodec("HGT")
    assert plain.adjust_longitude_range is False
    assert plain.to_dict() == {"name": "gribberish", "configuration": {"var": "HGT"}}
    assert GribberishCodec.from_dict(plain.to_dict()) == plain


HRRR = "hrrr.t06z.wrfsfcf01-TMP.grib2"
HRRR_SHAPE = (1059, 1799)


def test_codec_config_roundtrips_north_up_flag():
    from gribberish.zarr.codec import GribberishCodec

    codec = GribberishCodec("TMP", north_up=True)
    assert codec.to_dict() == {
        "name": "gribberish",
        "configuration": {"var": "TMP", "north_up": True},
    }
    assert GribberishCodec.from_dict(codec.to_dict()) == codec

    # both flags together round-trip independently
    both = GribberishCodec("TMP", adjust_longitude_range=True, north_up=True)
    assert both.to_dict() == {
        "name": "gribberish",
        "configuration": {
            "var": "TMP",
            "adjust_longitude_range": True,
            "north_up": True,
        },
    }
    assert GribberishCodec.from_dict(both.to_dict()) == both

    # default off omits the flag, so existing stored metadata round-trips unchanged
    plain = GribberishCodec("TMP")
    assert plain.north_up is False
    assert plain.to_dict() == {"name": "gribberish", "configuration": {"var": "TMP"}}
    assert GribberishCodec.from_dict(plain.to_dict()) == plain


async def test_north_up_flips_data_rows():
    """Opt-in north_up reverses the decoded data rows so row 0 is north-most.

    HRRR (Lambert) is south-first, so the flip is observable: north_up data
    equals a row-reversed native decode. Robust if HRRR were already north-first
    (then both branches assert a no-op)."""
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / HRRR).read_bytes()
    native = await _decode(GribberishCodec("TMP"), raw, HRRR_SHAPE)
    flipped = await _decode(GribberishCodec("TMP", north_up=True), raw, HRRR_SHAPE)

    lat_native = await _decode(GribberishCodec("latitude"), raw, HRRR_SHAPE)
    south_first = lat_native[0, 0] < lat_native[-1, 0]

    if south_first:
        np.testing.assert_array_equal(flipped, native[::-1, :])
        assert not np.array_equal(flipped, native)  # data really moved
    else:
        np.testing.assert_array_equal(flipped, native)  # already north-first: no-op


async def test_north_up_flips_latitude_coordinate():
    """With north_up the decoded latitude coord runs north-to-south (first row
    north of last)."""
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / HRRR).read_bytes()
    lat = await _decode(GribberishCodec("latitude", north_up=True), raw, HRRR_SHAPE)
    assert lat[0, 0] > lat[-1, 0]


async def test_north_up_noop_for_north_first_grid():
    """A north-first global grid (GEFS, lat 90..-90) is left untouched."""
    from gribberish.zarr.codec import GribberishCodec

    raw = (TEST_DATA / GEAVG).read_bytes()
    native = await _decode(GribberishCodec("HGT"), raw, GEAVG_SHAPE)
    flipped = await _decode(GribberishCodec("HGT", north_up=True), raw, GEAVG_SHAPE)
    np.testing.assert_array_equal(native, flipped)

    lat_native = await _decode(GribberishCodec("latitude"), raw, (GEAVG_SHAPE[0],))
    lat_flipped = await _decode(
        GribberishCodec("latitude", north_up=True), raw, (GEAVG_SHAPE[0],)
    )
    np.testing.assert_array_equal(lat_native, lat_flipped)


@pytest.mark.parametrize(
    "dtype_str",
    ["float64", "float32"],
)
async def test_decode_data_var_gribberish(dtype_str):
    from gribberish.zarr.codec import GribberishCodec
    from zarr.core.array_spec import ArraySpec, ArrayConfig
    from zarr.core.buffer import default_buffer_prototype
    from zarr.dtype import parse_dtype
    from zarr.buffer.cpu import buffer_prototype

    with open("./../test-data/hrrr.t06z.wrfsfcf01-UGRD.grib2", "rb") as f:
        raw_data = f.read()

    buffer = default_buffer_prototype().buffer.from_bytes(raw_data)
    codec = GribberishCodec(var="UGRD")
    decoded = await codec._decode_single(
        buffer,
        ArraySpec(
            shape=(1059, 1799),
            dtype=parse_dtype(dtype_str, zarr_format=3),
            fill_value=0,
            prototype=buffer_prototype,
            config=ArrayConfig(order="C", write_empty_chunks=False),
        ),
    )
    data = decoded.as_ndarray_like()

    assert data.shape == (1059, 1799)
    assert data.dtype == np.dtype(dtype_str)
    (
        np.testing.assert_almost_equal(data[0][1000], -4.46501350402832),
        "Data not decoded correctly",
    )

async def test_decode_chained_array_array_codec():
    """Test an array -> array codec (filter) chained after GribberishCodec"""
    from zarr.core.buffer import default_buffer_prototype

    from gribberish import parse_grib_array
    from gribberish.zarr.codec import GribberishCodec

    with open("./../test-data/hrrr.t06z.wrfsfcf01-TMP.grib2", "rb") as f:
        raw_data = f.read()

    store = zarr.storage.MemoryStore()
    array = zarr.create_array(
        store,
        name="tmp",
        shape=(1059, 1799),
        chunks=(1059, 1799),
        dtype="float64",
        fill_value=float("nan"),
        compressors=(),
        # decode order: serializer (GRIB bytes -> Kelvin), then filter (Kelvin -> Celsius)
        filters=[zarr.codecs.ScaleOffset(offset=-273.15, scale=1)],
        serializer=GribberishCodec(var="TMP"),
    )
    await store.set("tmp/c/0/0", default_buffer_prototype().buffer.from_bytes(raw_data))

    data = array[:]

    kelvin = parse_grib_array(raw_data, 0).reshape(1059, 1799)
    np.testing.assert_allclose(data, kelvin - 273.15)
