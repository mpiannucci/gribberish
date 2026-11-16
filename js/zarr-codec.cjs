// Shared implementation of the `gribberish` Zarr codec. `zarr.js` (Node) and
// `zarr.browser.js` (WebAssembly) bind it to their native `decodeZarrChunk`.

const CONFIG_KEYS = new Set(['var', 'adjust_longitude_range', 'north_up'])

function cStrides(shape) {
  const stride = Array.from({ length: shape.length }, () => 0)
  let step = 1
  for (let i = shape.length - 1; i >= 0; i--) {
    stride[i] = step
    step *= shape[i]
  }
  return stride
}

function createGribberishCodec(decodeZarrChunk) {
  /**
   * A read-only zarrita array-to-bytes codec that decodes each chunk (the raw
   * bytes of one GRIB message) with gribberish. Reads the same `"gribberish"`
   * codec metadata as the Python `gribberish.zarr.GribberishCodec` and the Rust
   * `zarrs` codec.
   */
  class GribberishCodec {
    constructor(configuration, meta) {
      const config = configuration ?? {}
      for (const key of Object.keys(config)) {
        if (!CONFIG_KEYS.has(key)) {
          throw new Error(`Unknown gribberish codec configuration key: ${key}`)
        }
      }
      if (meta.data_type !== 'float64' && meta.data_type !== 'float32') {
        throw new Error(`The gribberish codec only decodes to float32 or float64, not ${meta.data_type}`)
      }
      this.kind = 'array_to_bytes'
      this.configuration = {
        var: config.var ?? null,
        adjust_longitude_range: Boolean(config.adjust_longitude_range),
        north_up: Boolean(config.north_up),
      }
      this.dataType = meta.data_type
      this.shape = meta.shape
      this.stride = cStrides(meta.shape)
    }

    static fromConfig(configuration, meta) {
      return new GribberishCodec(configuration, meta)
    }

    encode() {
      throw new Error('The gribberish codec is read-only and does not support encoding')
    }

    decode(bytes) {
      const { var: variable, adjust_longitude_range, north_up } = this.configuration
      const values = decodeZarrChunk(bytes, variable, adjust_longitude_range, north_up)
      const expected = this.shape.reduce((a, b) => a * b, 1)
      if (values.length !== expected) {
        throw new Error(
          `GRIB message decoded to ${values.length} values, but the chunk shape [${this.shape}] holds ${expected}`,
        )
      }
      return {
        data: this.dataType === 'float32' ? new Float32Array(values) : values,
        shape: this.shape,
        stride: this.stride,
      }
    }
  }

  /** Register the codec under `"gribberish"` in a zarrita codec registry. */
  function registerGribberishCodec(registry) {
    registry.set('gribberish', () => Promise.resolve(GribberishCodec))
  }

  return { GribberishCodec, registerGribberishCodec }
}

module.exports = { createGribberishCodec }
