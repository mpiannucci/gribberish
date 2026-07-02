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
    """Transform GRIB2 bytes into zarr arrays using gribberish library

    When ``adjust_longitude_range`` is set, global 0–360° longitude grids are
    rewrapped to a monotonic −180…180° range: the decoded data is rolled along
    the longitude axis and the ``longitude`` coordinate is wrapped to match, so
    label-based slicing across the prime meridian behaves as expected. It is a
    no-op for grids that don't span the globe.

    When ``north_up`` is set, the decoded data rows and the latitude/y coordinate
    are reordered so the 0th row is the northern-most. It is a no-op for grids
    that are already north-first.
    """

    var: str | None
    adjust_longitude_range: bool = False
    north_up: bool = False

    def __init__(
        self,
        var: str | None,
        adjust_longitude_range: bool = False,
        north_up: bool = False,
    ) -> Self:
        object.__setattr__(self, "var", var)
        object.__setattr__(self, "adjust_longitude_range", bool(adjust_longitude_range))
        object.__setattr__(self, "north_up", bool(north_up))

    @classmethod
    def from_dict(cls, data: dict[str, JSON]) -> Self:
        _, configuration_parsed = parse_named_configuration(
            data, "gribberish", require_configuration=False
        )
        configuration_parsed = configuration_parsed or {}
        return cls(**configuration_parsed)  # type: ignore[arg-type]

    def to_dict(self) -> dict[str, JSON]:
        configuration: dict[str, JSON] = {}
        if self.var:
            configuration["var"] = self.var
        if self.adjust_longitude_range:
            configuration["adjust_longitude_range"] = True
        if self.north_up:
            configuration["north_up"] = True
        if not configuration:
            return {"name": "gribberish"}
        return {"name": "gribberish", "configuration": configuration}

    async def _decode_single(
        self,
        chunk_data: Buffer,
        chunk_spec: ArraySpec,
    ) -> NDBuffer:
        assert isinstance(chunk_data, Buffer)
        chunk_bytes = chunk_data.to_bytes()

        if self.var == 'latitude' or self.var == 'longitude':
            message = parse_grib_message_metadata(chunk_bytes, 0)
            lat, lng = message.latlng(self.adjust_longitude_range, self.north_up)
            data: NDArrayLike = lat if self.var == 'latitude' else lng
        else:
            data: NDArrayLike = parse_grib_array(
                chunk_bytes, 0, self.adjust_longitude_range, self.north_up
            )

        if (native_dtype := chunk_spec.dtype.to_native_dtype()) != data.dtype:
            data = data.astype(native_dtype)
        if data.shape != chunk_spec.shape:
            data = data.reshape(chunk_spec.shape)

        return chunk_spec.prototype.nd_buffer.from_ndarray_like(data)

    async def _encode_single(
        self,
        chunk_data: NDBuffer,
        chunk_spec: ArraySpec,
    ) -> Buffer | None:
        # This is a read-only codec
        raise NotImplementedError

register_codec("gribberish", GribberishCodec)
