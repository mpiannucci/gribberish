import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join, dirname } from 'node:path'

import test from 'ava'

import {
  adjustLatitudeValues,
  adjustLongitudeValues,
  GribMessage,
  GribMessageFactory,
  GribMessageMetadataFactory,
  parseGribIndex,
  parseMessagesFromBuffer,
} from '../index'

const DATA_DIR = join(dirname(fileURLToPath(import.meta.url)), '../../test-data')

// GEFS 0.5° — first message (HGT) is a 361×720 global grid, lon 0..359.5. The
// antimeridian (180°) sits on column 360, so the wrap rolls by exactly 360.
const GEAVG = 'geavg.t12z.pgrb2a.0p50.f000'
const GEAVG_COLS = 720
const GEAVG_ROLL = 360

// HRRR — a south-first Lambert Conformal grid.
const HRRR = 'hrrr.t06z.wrfsfcf01-TMP.grib2'

test('parseMessagesFromBuffer reads HRRR GRIB2 messages', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const messages = parseMessagesFromBuffer(data)

  t.true(messages.length > 0, 'should read at least one message')

  const first = messages[0]
  t.truthy(first.varName)
  t.truthy(first.varAbbrev)
  t.truthy(first.units)
  t.truthy(first.proj)
  t.truthy(first.crs)
  t.true(first.gridShape.rows > 0)
  t.true(first.gridShape.cols > 0)
  t.true(first.data.length > 0)
  t.is(first.data.length, first.gridShape.rows * first.gridShape.cols)
})

test('GribMessage.parseFromBuffer parses a single message', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const msg = GribMessage.parseFromBuffer(data, 0)

  t.truthy(msg.key)
  t.truthy(msg.varName)
  t.truthy(msg.varAbbrev)
  t.truthy(msg.units)
  t.truthy(msg.forecastDate)
  t.truthy(msg.referenceDate)
  t.truthy(msg.proj)
  t.truthy(msg.crs)
  t.true(msg.gridShape.rows > 0)
  t.true(msg.gridShape.cols > 0)
  t.true(msg.latlng.latitude.length > 0)
  t.true(msg.latlng.longitude.length > 0)
  t.true(msg.data.length > 0)
})

test('GribMessage latlng and data match grid shape', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const msg = GribMessage.parseFromBuffer(data, 0)
  const expectedPoints = msg.gridShape.rows * msg.gridShape.cols

  t.is(msg.latlng.latitude.length, expectedPoints)
  t.is(msg.latlng.longitude.length, expectedPoints)
  t.is(msg.data.length, expectedPoints)
})

test('GribMessageFactory lists and retrieves available messages', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const factory = GribMessageFactory.fromBuffer(data)

  const keys = factory.availableMessages
  t.true(keys.length > 0, 'should have available messages')

  const firstKey = keys[0]
  const msg = factory.getMessage(firstKey)
  t.truthy(msg.varName)
  t.true(msg.data.length > 0)
})

test('GribMessageMetadataFactory lists and retrieves messages efficiently', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const factory = GribMessageMetadataFactory.fromBuffer(data)

  const keys = factory.availableMessages
  t.true(keys.length > 0, 'should have available messages')

  const firstKey = keys[0]
  const msg = factory.getMessage(firstKey)
  t.truthy(msg.varName)
  t.true(msg.data.length > 0)
})

test('parseGribIndex locates messages for ranged reads', (t) => {
  // NOAA .idx: one entry per message, lengths inferred from the next offset.
  const idxText = readFileSync(join(DATA_DIR, 'gfswave.t18z.atlocn.0p16.f001.grib2.idx'), 'utf8')
  const grib = readFileSync(join(DATA_DIR, 'gfswave.t18z.atlocn.0p16.f001.grib2'))
  const entries = parseGribIndex(idxText, grib.length)

  t.is(entries.length, 19)
  t.like(entries[0], { var: 'WIND', offset: 0, length: 41723, level: 'surface' })

  // A byte range sliced via an index entry parses as a standalone message —
  // the fetch + Range-header pattern. (HRRR fixture: complex packing, which
  // also decodes on the wasm32 build; the wave file is JPEG2000, which doesn't.)
  const hrrr = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const [entry] = parseGribIndex('1:0:d=2023072206:TMP:2 m above ground:1 hour fcst:', hrrr.length)
  const msg = GribMessage.parseFromBuffer(hrrr.subarray(entry.offset, entry.offset + (entry.length ?? 0)), 0)
  t.is(msg.varAbbrev, entry.var)

  // ECMWF .index: explicit lengths, MARS keys verbatim.
  const ecmwf = parseGribIndex(
    '{"domain": "g", "date": "20260610", "time": "0000", "step": "3", "levtype": "sfc", "param": "2t", "_offset": 0, "_length": 224}',
  )
  t.like(ecmwf[0], { var: '2t', offset: 0, length: 224, forecastTime: '3' })
  t.is(ecmwf[0].keys.levtype, 'sfc')
})

test('latlngAdjusted wraps a global 0..360 grid to monotonic [-180, 180)', (t) => {
  const data = readFileSync(join(DATA_DIR, GEAVG))
  const msg = parseMessagesFromBuffer(data)[0]

  const native = msg.latlng
  const wrapped = msg.latlngAdjusted(true, false)

  // Default longitude axis is the native 0..360; latitudes are untouched.
  t.is(native.longitude[0], 0.0)
  t.is(native.longitude[GEAVG_COLS - 1], 359.5)
  t.deepEqual(wrapped.latitude, native.latitude)

  // Wrapped longitude axis is strictly monotonic over [-180, 180).
  const wrappedLon = wrapped.longitude.slice(0, GEAVG_COLS)
  t.is(wrappedLon[0], -180.0)
  t.is(wrappedLon[GEAVG_COLS - 1], 179.5)
  for (let i = 1; i < wrappedLon.length; i++) {
    t.true(wrappedLon[i] > wrappedLon[i - 1], `not monotonic at index ${i}`)
  }
  t.true(wrappedLon.every((lon) => lon >= -180.0 && lon < 180.0))

  // adjustLongitudeRange=false is identical to the plain getter.
  t.deepEqual(msg.latlngAdjusted(false, false).longitude, native.longitude)
})

test('dataAdjusted rolls a global grid left by the split column', (t) => {
  const data = readFileSync(join(DATA_DIR, GEAVG))
  const msg = parseMessagesFromBuffer(data)[0]

  const native = msg.data
  const adjusted = msg.dataAdjusted(true, false)
  const rows = msg.gridShape.rows
  const cols = msg.gridShape.cols
  t.is(cols, GEAVG_COLS)

  // Each row is rotated left by GEAVG_ROLL columns, matching the wrapped axis.
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      t.is(adjusted[r * cols + c], native[r * cols + ((c + GEAVG_ROLL) % cols)])
    }
  }

  // Default and explicit-off both return the data unmoved.
  t.deepEqual(msg.dataAdjusted(false, false), native)
})

test('adjusted accessors are a no-op for a non-global (HRRR Lambert) grid', (t) => {
  const data = readFileSync(join(DATA_DIR, HRRR))
  const msg = GribMessage.parseFromBuffer(data, 0)

  // The longitude wrap is a no-op on a non-global grid.
  t.deepEqual(msg.latlngAdjusted(true, false).longitude, msg.latlng.longitude)
  t.deepEqual(msg.dataAdjusted(true, false), msg.data)
})

test('northUp puts the northern-most row first on an HRRR Lambert grid', (t) => {
  const data = readFileSync(join(DATA_DIR, HRRR))
  const msg = GribMessage.parseFromBuffer(data, 0)

  const rows = msg.gridShape.rows
  const cols = msg.gridShape.cols
  const nativeData = msg.data
  const nativeLat = msg.latlng.latitude

  const adjustedData = msg.dataAdjusted(false, true)
  const adjustedLat = msg.latlngAdjusted(false, true).latitude

  // The fixture is south-first (row 0 south of the last), so the flip moves.
  t.true(nativeLat[0] < nativeLat[(rows - 1) * cols])

  // northUp reverses whole rows of both data and coordinates.
  const expectedData = new Array<number>(nativeData.length)
  const expectedLat = new Array<number>(nativeLat.length)
  for (let r = 0; r < rows; r++) {
    const src = (rows - 1 - r) * cols
    for (let c = 0; c < cols; c++) {
      expectedData[r * cols + c] = nativeData[src + c]
      expectedLat[r * cols + c] = nativeLat[src + c]
    }
  }
  t.deepEqual(adjustedData, expectedData)
  t.deepEqual(adjustedLat, expectedLat)
  t.true(adjustedLat[0] > adjustedLat[(rows - 1) * cols])

  // northUp=false leaves both untouched.
  t.deepEqual(msg.dataAdjusted(false, false), nativeData)
  t.deepEqual(msg.latlngAdjusted(false, false).latitude, nativeLat)
})

test('northUp is a no-op on an already-north-first global grid', (t) => {
  const data = readFileSync(join(DATA_DIR, GEAVG))
  const msg = parseMessagesFromBuffer(data)[0]

  const nativeLat = msg.latlng.latitude
  // GEFS fixtures are north-first (lat 90 -> -90), so north_up does nothing.
  t.true(nativeLat[0] > nativeLat[nativeLat.length - 1])
  t.deepEqual(msg.dataAdjusted(false, true), msg.data)
  t.deepEqual(msg.latlngAdjusted(false, true).latitude, nativeLat)

  // Composable with the longitude wrap: northUp leaves the wrap result alone.
  t.deepEqual(msg.dataAdjusted(true, true), msg.dataAdjusted(true, false))
  t.deepEqual(msg.latlngAdjusted(true, true).longitude, msg.latlngAdjusted(true, false).longitude)
})

test('northUp is optional: the one-arg form stays backward compatible', (t) => {
  const data = readFileSync(join(DATA_DIR, GEAVG))
  const msg = parseMessagesFromBuffer(data)[0]

  // Omitting northUp defaults it to false.
  t.deepEqual(msg.dataAdjusted(true), msg.dataAdjusted(true, false))
  t.deepEqual(msg.latlngAdjusted(true).longitude, msg.latlngAdjusted(true, false).longitude)
  t.deepEqual(msg.latlngAdjusted(true).latitude, msg.latlngAdjusted(true, false).latitude)
})

test('adjustLongitudeValues wraps a global axis and leaves a regional one unchanged', (t) => {
  // Global 0..359.5 axis wraps to a monotonic [-180, 180) axis.
  const global = Array.from({ length: GEAVG_COLS }, (_, i) => i * 0.5)
  const wrapped = adjustLongitudeValues(global)
  t.is(wrapped[0], -180.0)
  t.is(wrapped[GEAVG_COLS - 1], 179.5)
  for (let i = 1; i < wrapped.length; i++) {
    t.true(wrapped[i] > wrapped[i - 1])
  }

  // A regional subset (not near-global) is returned unchanged.
  const regional = Array.from({ length: 100 }, (_, i) => 200 + i * 0.25)
  t.deepEqual(adjustLongitudeValues(regional), regional)
})

test('adjustLatitudeValues reverses an ascending axis and leaves a descending one unchanged', (t) => {
  // Ascending (south-first) axis is reversed to descend north -> south.
  const ascending = Array.from({ length: 181 }, (_, i) => -90 + i)
  const reversed = adjustLatitudeValues(ascending)
  t.is(reversed[0], 90)
  t.is(reversed[reversed.length - 1], -90)
  for (let i = 1; i < reversed.length; i++) {
    t.true(reversed[i] < reversed[i - 1])
  }

  // Already-descending (north-first) axis is returned unchanged.
  const descending = Array.from({ length: 181 }, (_, i) => 90 - i)
  t.deepEqual(adjustLatitudeValues(descending), descending)
})

test('GribMessageFactory throws for unknown key', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const factory = GribMessageFactory.fromBuffer(data)

  t.throws(() => factory.getMessage('nonexistent_key_xyz'), {
    message: /not found/,
  })
})
