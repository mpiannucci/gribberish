/** The `"gribberish"` codec configuration, shared with the Python and Rust codecs. */
export interface GribberishCodecConfiguration {
  /** `"latitude"` or `"longitude"` decodes that coordinate instead of the data values. */
  var?: string | null
  /** Rewrap global 0–360° longitude grids to a monotonic −180…180° range. */
  adjust_longitude_range?: boolean
  /** Reorder rows so the 0th row is the northern-most. */
  north_up?: boolean
}

export interface GribberishChunk<T extends Float32Array | Float64Array = Float32Array | Float64Array> {
  data: T
  shape: number[]
  stride: number[]
}

/**
 * A read-only zarrita array-to-bytes codec that decodes each chunk (the raw
 * bytes of one GRIB message) with gribberish. Reads the same `"gribberish"`
 * codec metadata as the Python and Rust codecs, so virtual stores written by
 * the gribberish VirtualiZarr parser can be read from JavaScript.
 */
export declare class GribberishCodec {
  readonly kind: 'array_to_bytes'
  readonly configuration: Required<GribberishCodecConfiguration>
  constructor(configuration: GribberishCodecConfiguration | undefined, meta: { data_type: string; shape: number[] })
  static fromConfig(
    configuration: GribberishCodecConfiguration | undefined,
    meta: { data_type: string; shape: number[] },
  ): GribberishCodec
  /** Always throws: the codec is read-only. */
  encode(): never
  decode(bytes: Uint8Array): GribberishChunk
}

/**
 * Register the codec under `"gribberish"` in a zarrita codec registry:
 *
 * ```ts
 * import * as zarr from 'zarrita'
 * import { registerGribberishCodec } from '@mattnucc/gribberish/zarr'
 *
 * registerGribberishCodec(zarr.registry)
 * ```
 */
export declare function registerGribberishCodec(registry: Map<string, () => Promise<unknown>>): void
