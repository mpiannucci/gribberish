from dataclasses import dataclass
from typing import Self

from gribberish import parse_grib_array, parse_grib_message_metadata
from zarr.abc.codec import ArrayBytesCodec
from zarr.core.array_spec import ArraySpec
from zarr.core.buffer import Buffer, NDArrayLike, NDBuffer
from zarr.core.common import JSON, parse_named_configuration
from zarr.registry import register_codec


@dataclass(frozen=True)
class GribberishCodec(ArrayBytesCodec):
    """Transform GRIB2 bytes into zarr arrays using gribberish library"""

    var: str | None

    def __init__(self, var: str | None) -> Self:
        object.__setattr__(self, "var", var)

    @classmethod
    def from_dict(cls, data: dict[str, JSON]) -> Self:
        _, configuration_parsed = parse_named_configuration(
            data, "gribberish", require_configuration=False
        )
        configuration_parsed = configuration_parsed or {}
        return cls(**configuration_parsed)  # type: ignore[arg-type]

    def to_dict(self) -> dict[str, JSON]:
        if not self.var:
            return {"name": "gribberish"}
        else:
            return {"name": "gribberish", "configuration": {"var": self.var}}

    async def _decode_single(
        self,
        chunk_data: Buffer,
        chunk_spec: ArraySpec,
    ) -> NDBuffer:
        assert isinstance(chunk_data, Buffer)
        chunk_bytes = chunk_data.to_bytes()

        if self.var == 'latitude' or self.var == 'longitude':
            message = parse_grib_message_metadata(chunk_bytes, 0)
            lat, lng = message.latlng()
            data: NDArrayLike = lat if self.var == 'latitude' else lng
        else:
            data: NDArrayLike = parse_grib_array(chunk_bytes, 0)

        if (native_dtype := chunk_spec.dtype.to_native_dtype()) != data.dtype:
            data = data.astype(native_dtype)
        if data.shape != chunk_spec.shape:
            data = data.reshape(chunk_spec.shape)

        return data

    async def _encode_single(
        self,
        chunk_data: NDBuffer,
        chunk_spec: ArraySpec,
    ) -> Buffer | None:
        # This is a read-only codec
        raise NotImplementedError

register_codec("gribberish", GribberishCodec)
