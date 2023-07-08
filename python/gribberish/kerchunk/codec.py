import numcodecs

from ..gribberishpy import parse_grib_array


class GribberishCodec(numcodecs.abc.Codec):
    """
    Read GRIB stream of bytes as a message using gribberish

    Adapted from https://github.com/fsspec/kerchunk/blob/main/kerchunk/codecs.py
    """

    # eclock = threading.RLock()

    codec_id = "gribberish"

    def __init__(self, var, shape, dtype=None):
        self.var = var
        self.shape = shape
        self.dtype = dtype

    def encode(self, buf):
        # on encode, pass through
        return buf

    def decode(self, buf, out=None):
        data = parse_grib_array(buf, 0, self.shape)
        if out is not None:
            return numcodecs.compat.ndarray_copy(data, out)
        else:
            return data.astype(self.dtype)


numcodecs.register_codec(GribberishCodec, "gribberish")