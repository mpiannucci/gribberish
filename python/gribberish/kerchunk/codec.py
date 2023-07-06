import numcodecs

from ..gribberishpy import parse_grib_data


class GribberishCodec(numcodecs.abc.Codec):
    """
    Read GRIB stream of bytes as a message using gribberish

    Adapted from https://github.com/fsspec/kerchunk/blob/main/kerchunk/codecs.py
    """

    # eclock = threading.RLock()

    codec_id = "gribberish"

    def __init__(self, var, dtype=None):
        self.var = var
        self.dtype = dtype

    def encode(self, buf):
        # on encode, pass through
        return buf

    def decode(self, buf, out=None):
        data = parse_grib_data(buf, 0)
        if out is not None:
            return numcodecs.compat.ndarray_copy(data, out)
        else:
            return data.astype(self.dtype)


numcodecs.register_codec(GribberishCodec, "gribberish")