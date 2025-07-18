import pytest

import numpy as np

zarr = pytest.importorskip("zarr")


async def test_decode_data_var_gribberish():
    from gribberish.zarr.codec import GribberishCodec
    from zarr.core.array_spec import ArraySpec
    from zarr.core.buffer import default_buffer_prototype

    with open("./../gribberish/tests/data/hrrr.t06z.wrfsfcf01-UGRD.grib2", "rb") as f:
        raw_data = f.read()

    buffer = default_buffer_prototype().buffer.from_bytes(raw_data)
    codec = GribberishCodec(var="UGRD")
    data = await codec._decode_single(
        buffer,
        ArraySpec(
            shape=(1059, 1799),
            dtype="float64",
            fill_value=0,
            order="C",
            prototype=np.ndarray,
        ),
    )

    assert data.shape == (1059, 1799)
    assert data.dtype == np.dtype("float64")
    (
        np.testing.assert_almost_equal(data[0][1000], -4.46501350402832),
        "Data not decoded correctly",
    )
