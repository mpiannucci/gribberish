import pytest

import numpy as np

zarr = pytest.importorskip("zarr")


@pytest.mark.parametrize(
    'dtype_str',
    ['float64', 'float32'],
)
async def test_decode_data_var_gribberish(dtype_str):
    from gribberish.zarr.codec import GribberishCodec
    from zarr.core.array_spec import ArraySpec, ArrayConfig
    from zarr.core.buffer import default_buffer_prototype
    from zarr.dtype import parse_dtype
    from zarr.buffer.cpu import buffer_prototype

    with open("./../gribberish/tests/data/hrrr.t06z.wrfsfcf01-UGRD.grib2", "rb") as f:
        raw_data = f.read()

    buffer = default_buffer_prototype().buffer.from_bytes(raw_data)
    codec = GribberishCodec(var="UGRD")
    data = await codec._decode_single(
        buffer,
        ArraySpec(
            shape=(1059, 1799),
            dtype=parse_dtype(dtype_str, zarr_format=3),
            fill_value=0,
            prototype=buffer_prototype,
            config=ArrayConfig(order='C', write_empty_chunks=False),
        ),
    )

    assert data.shape == (1059, 1799)
    assert data.dtype == np.dtype(dtype_str)
    (
        np.testing.assert_almost_equal(data[0][1000], -4.46501350402832),
        "Data not decoded correctly",
    )
