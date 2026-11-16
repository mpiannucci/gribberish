import { decodeZarrChunk } from '@mattnucc/gribberish-wasm32-wasi'
import codec from './zarr-codec.cjs'

export const { GribberishCodec, registerGribberishCodec } = codec.createGribberishCodec(decodeZarrChunk)
