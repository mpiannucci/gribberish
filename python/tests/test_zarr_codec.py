import pytest

import numpy as np

zarr = pytest.importorskip("zarr")


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
