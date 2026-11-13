import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join, dirname } from 'node:path'

import test from 'ava'
import * as zarr from 'zarrita'

import { GribMessage } from '../index'
import { GribberishCodec, registerGribberishCodec } from '../zarr'

const DATA_DIR = join(dirname(fileURLToPath(import.meta.url)), '../../test-data')

// GEFS 0.5° — first message (HGT) is a 361×720 global grid, lon 0..359.5.
const GEAVG = 'geavg.t12z.pgrb2a.0p50.f000'
const GEAVG_SHAPE = [361, 720]
const GEAVG_ROLL = 360

// HRRR — a south-first Lambert Conformal grid.
const HRRR = 'hrrr.t06z.wrfsfcf01-TMP.grib2'

registerGribberishCodec(zarr.registry)

/** Split a GRIB2 file into its messages' raw bytes using each header's total length. */
function messages(name: string): Uint8Array[] {
  const data = new Uint8Array(readFileSync(join(DATA_DIR, name)))
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength)
  const out: Uint8Array[] = []
  for (let offset = 0; offset + 16 <= data.length; ) {
    const length = Number(view.getBigUint64(offset + 8))
    out.push(data.subarray(offset, offset + length))
    offset += length
  }
  return out
}

/** An in-memory store holding a single gribberish-encoded array at `/arr`. */
function store(
  configuration: Record<string, unknown> | undefined,
  shape: number[],
  chunkShape: number[],
  dataType: string,
  chunks: Record<string, Uint8Array>,
) {
  const codec: Record<string, unknown> = { name: 'gribberish' }
  if (configuration) codec.configuration = configuration
  const metadata = {
    zarr_format: 3,
    node_type: 'array',
    shape,
    data_type: dataType,
    chunk_grid: { name: 'regular', configuration: { chunk_shape: chunkShape } },
    chunk_key_encoding: { name: 'default', configuration: { separator: '/' } },
    fill_value: 'NaN',
    codecs: [codec],
    attributes: {},
  }
  const files = new Map<string, Uint8Array>([['/arr/zarr.json', new TextEncoder().encode(JSON.stringify(metadata))]])
  for (const [key, bytes] of Object.entries(chunks)) files.set(`/arr/c/${key}`, bytes)
  return { get: async (key: string) => files.get(key) }
}

async function readChunk(
  configuration: Record<string, unknown> | undefined,
  shape: number[],
  dataType: string,
  chunk: Uint8Array,
) {
  const s = store(configuration, shape, shape, dataType, { [shape.map(() => '0').join('/')]: chunk })
  const arr = await zarr.open.v3(zarr.root(s).resolve('arr'), { kind: 'array' })
  return arr.getChunk(shape.map(() => 0))
}

test('decodes data like GribMessage.data', async (t) => {
  const [raw] = messages(GEAVG)
  const expected = GribMessage.parseFromBuffer(raw, 0).data

  const chunk = await readChunk({ var: 'HGT' }, GEAVG_SHAPE, 'float64', raw)

  t.true(chunk.data instanceof Float64Array)
  t.deepEqual(chunk.shape, GEAVG_SHAPE)
  t.deepEqual(Array.from(chunk.data as Float64Array), expected)
})

test('decodes without a configuration', async (t) => {
  const [raw] = messages(GEAVG)
  const expected = GribMessage.parseFromBuffer(raw, 0).data

  const chunk = await readChunk(undefined, GEAVG_SHAPE, 'float64', raw)

  t.deepEqual(Array.from(chunk.data as Float64Array), expected)
})

test('casts to float32', async (t) => {
  const [raw] = messages(GEAVG)
  const expected = Float32Array.from(GribMessage.parseFromBuffer(raw, 0).data)

  const chunk = await readChunk({ var: 'HGT' }, GEAVG_SHAPE, 'float32', raw)

  t.true(chunk.data instanceof Float32Array)
  t.deepEqual(chunk.data, expected)
})

test('decodes 1-D latitude and longitude axes of a regular grid', async (t) => {
  const [raw] = messages(GEAVG)
  const { latitude, longitude } = GribMessage.parseFromBuffer(raw, 0).latlng

  const lat = await readChunk({ var: 'latitude' }, [GEAVG_SHAPE[0]], 'float64', raw)
  const lng = await readChunk({ var: 'longitude' }, [GEAVG_SHAPE[1]], 'float64', raw)

  t.deepEqual(Array.from(lat.data as Float64Array), latitude)
  t.deepEqual(Array.from(lng.data as Float64Array), longitude)
})

test('adjust_longitude_range rolls data and wraps longitudes', async (t) => {
  const [raw] = messages(GEAVG)
  const msg = GribMessage.parseFromBuffer(raw, 0)
  const config = { var: 'HGT', adjust_longitude_range: true }

  const data = await readChunk(config, GEAVG_SHAPE, 'float64', raw)
  const lng = await readChunk({ ...config, var: 'longitude' }, [GEAVG_SHAPE[1]], 'float64', raw)

  const values = data.data as Float64Array
  t.deepEqual(Array.from(values), msg.dataAdjusted(true))
  // Column 0 of the adjusted grid is column GEAVG_ROLL of the raw grid.
  t.is(values[0], msg.data[GEAVG_ROLL])
  t.deepEqual(Array.from(lng.data as Float64Array), msg.latlngAdjusted(true).longitude)
  t.is((lng.data as Float64Array)[0], -180)
})

test('north_up flips a south-first grid', async (t) => {
  const [raw] = messages(HRRR)
  const msg = GribMessage.parseFromBuffer(raw, 0)
  const shape = [msg.gridShape.rows, msg.gridShape.cols]
  const config = { var: 'TMP', north_up: true }

  const data = await readChunk(config, shape, 'float64', raw)
  // A projected grid's coordinates are 2-D, one per grid cell.
  const lat = await readChunk({ ...config, var: 'latitude' }, shape, 'float64', raw)

  t.deepEqual(Array.from(data.data as Float64Array), msg.dataAdjusted(false, true))
  const adjustedLat = msg.latlngAdjusted(false, true).latitude
  t.deepEqual(Array.from(lat.data as Float64Array), adjustedLat)
  t.true(adjustedLat[0] > adjustedLat[adjustedLat.length - 1])
})

test('reads a multi-chunk array with one message per chunk', async (t) => {
  const raws = messages(GEAVG).slice(0, 2)
  const s = store({}, [2, ...GEAVG_SHAPE], [1, ...GEAVG_SHAPE], 'float32', {
    '0/0/0': raws[0],
    '1/0/0': raws[1],
  })
  const arr = await zarr.open.v3(zarr.root(s).resolve('arr'), { kind: 'array' })

  // A point read across both chunks (time series at one grid cell).
  const point = await zarr.get(arr, [null, 100, 200])
  const index = 100 * GEAVG_SHAPE[1] + 200
  const expected = raws.map((raw) => Math.fround(GribMessage.parseFromBuffer(raw, 0).data[index]))
  t.deepEqual(Array.from(point.data as Float32Array), expected)
})

test('rejects a chunk shape that does not match the message', async (t) => {
  const [raw] = messages(GEAVG)
  await t.throwsAsync(() => readChunk({ var: 'HGT' }, [10, 10], 'float64', raw), {
    message: /decoded to 259920 values, but the chunk shape \[10,10\] holds 100/,
  })
})

test('rejects unsupported data types and unknown configuration', (t) => {
  t.throws(() => GribberishCodec.fromConfig({}, { data_type: 'int32', shape: [1] }), {
    message: /only decodes to float32 or float64/,
  })
  t.throws(() => GribberishCodec.fromConfig({ nope: 1 } as any, { data_type: 'float64', shape: [1] }), {
    message: /Unknown gribberish codec configuration key: nope/,
  })
})

test('is read-only', (t) => {
  const codec = GribberishCodec.fromConfig({}, { data_type: 'float64', shape: [1] })
  t.throws(() => codec.encode(), { message: /read-only/ })
})

test('surfaces invalid GRIB bytes as errors', async (t) => {
  await t.throwsAsync(() => readChunk({}, [2, 2], 'float64', new Uint8Array(32)), {
    message: /Failed to read GRIB message/,
  })
})
