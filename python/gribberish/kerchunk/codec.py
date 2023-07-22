import numcodecs

from ..gribberishpy import parse_grib_array, parse_grib_message_metadata


class GribberishCodec(numcodecs.abc.Codec):
    """
    Read GRIB stream of bytes as a message using gribberish

    Adapted from https://github.com/fsspec/kerchunk/blob/main/kerchunk/codecs.py
    """

    codec_id = "gribberish"

    def __init__(self, var, dtype=None):
        self.var = var
        self.dtype = dtype

    def encode(self, buf):
        # on encode, pass through
        return buf

    def decode(self, buf, out=None):
        if self.var == 'latitude' or self.var == 'longitude':
            message = parse_grib_message_metadata(buf, 0)
            lat, lng = message.latlng()
            data = lat if self.var == 'latitude' else lng
        else:
            data = parse_grib_array(buf, 0)

        if out is not None:
            return numcodecs.compat.ndarray_copy(data, out)
        else:
            return data.astype(self.dtype)


numcodecs.register_codec(GribberishCodec, "gribberish")