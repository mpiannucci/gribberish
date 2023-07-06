import base64
import fsspec
import zarr
import numpy as np

from kerchunk.utils import class_factory, _encode_for_JSON
from .codec import GribberishCodec
from ..gribberishpy import parse_grib_mapping, parse_grid_dataset


def _split_file(f, skip=0):
    if hasattr(f, "size"):
        size = f.size
    else:
        f.seek(0, 2)
        size = f.tell()
        f.seek(0)
    part = 0

    while f.tell() < size:
        start = f.tell()
        head = f.read(16)
        marker = head[:4]
        if not marker:
            break  # EOF
        assert head[:4] == b"GRIB", "Bad grib message start marker"
        part_size = int.from_bytes(head[12:], "big")
        f.seek(start)
        yield start, part_size, f.read(part_size)
        part += 1
        if skip and part >= skip:
            break


def _store_array_inline(store, z, data, var, attr):
    shape = tuple(data.shape or ())
    d = z.create_dataset(
        name=var,
        shape=shape,
        chunks=shape,
        dtype=data.dtype,
        compressor=False,
    )
    if hasattr(data, "tobytes"):
        b = data.tobytes()
    else:
        b = data.build_array().tobytes()
    try:
        # easiest way to test if data is ascii
        b.decode("ascii")
    except UnicodeDecodeError:
        b = b"base64:" + base64.b64encode(data)
    store[f"{var}/0"] = b.decode("ascii")
    d.attrs.update(attr)


def _store_array_ref(
    store,
    z,
    data_shape,
    var,
    offset,
    size,
    attr
):
    shape = tuple(data_shape or ())
    data_type = np.dtype('float64')
    d = z.create_dataset(
        name=var,
        shape=shape,
        chunks=shape,
        dtype=data_type,
        filters=[GribberishCodec(var=var, dtype=str(data_type))],
        compressor=False,
        overwrite=True,
    )
    store[f"{var}/" + ".".join(["0"] * len(shape))] = ["{{u}}", offset, size]
    d.attrs.update(attr)


def scan_gribberish(
    url,
    common=None,
    storage_options=None,
    skip=0,
    only_vars=None,
):
    """
    Generate references for a GRIB2 file using gribberish

    Parameters
    ----------

    url: str
        File location
    common_vars: (depr, do not use)
    storage_options: dict
        For accessing the data, passed to filesystem
    skip: int
        If non-zero, stop processing the file after this many messages
    only_vars: list(str)
        If given, only store these variables

    Returns
    -------

    list(dict): references dicts in Version 1 format, one per message
    """
    import eccodes

    storage_options = storage_options or {}

    out = []
    with fsspec.open(url, "rb", **storage_options) as f:
        for offset, size, data in _split_file(f, skip=skip):
            dataset = parse_grid_dataset(data)

            # Only reading one variable from each data chunk (1 message)
            var_name, var_data = next(iter(dataset['data_vars'].items()))
            if only_vars and var_name not in only_vars:
                continue

            store = {}
            z = zarr.open_group(store)
            z.attrs.update(dataset['attrs'])

            _store_array_ref(
                store,
                z,
                var_data['values']['shape'],
                var_name,
                offset,
                size,
                var_data['attrs']
            )

            # Coords
            dims = var_data['dims']
            z[var_name].attrs["_ARRAY_DIMENSIONS"] = dims

            for coord_name, coord_data in dataset['coords'].items():
                # TODO: Prob dont store inline for non regular grids
                _store_array_inline(
                    store,
                    z,
                    np.array(coord_data['values']),
                    coord_name,
                    coord_data['attrs']
                )
                z[coord_name].attrs["_ARRAY_DIMENSIONS"] = dims

            out.append(
                {
                    "version": 1,
                    "refs": _encode_for_JSON(store),
                    "templates": {"u": url},
                }
            )
    return out


GribberishToZarr = class_factory(scan_gribberish)
