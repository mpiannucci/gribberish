const { decodeZarrChunk } = require('./index.js')
const { createGribberishCodec } = require('./zarr-codec.cjs')

const codec = createGribberishCodec(decodeZarrChunk)

// Assigned one by one so Node can detect the named exports for ESM imports.
module.exports.GribberishCodec = codec.GribberishCodec
module.exports.registerGribberishCodec = codec.registerGribberishCodec
