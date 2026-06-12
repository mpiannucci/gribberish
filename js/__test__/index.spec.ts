import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join, dirname } from 'node:path'

import test from 'ava'

import { GribMessage, GribMessageFactory, GribMessageMetadataFactory, parseGribIndex, parseMessagesFromBuffer } from '../index'

const DATA_DIR = join(dirname(fileURLToPath(import.meta.url)), '../../test-data')

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
  // NOAA .idx: entries carry byte ranges, and a range sliced out of the file
  // parses as a standalone message — the fetch + Range-header pattern.
  const idxText = readFileSync(join(DATA_DIR, 'gfswave.t18z.atlocn.0p16.f001.grib2.idx'), 'utf8')
  const grib = readFileSync(join(DATA_DIR, 'gfswave.t18z.atlocn.0p16.f001.grib2'))
  const entries = parseGribIndex(idxText, grib.length)

  t.is(entries.length, 19)
  t.like(entries[0], { var: 'WIND', offset: 0, length: 41723, level: 'surface' })

  const entry = entries[2]
  const msg = GribMessage.parseFromBuffer(grib.subarray(entry.offset, entry.offset + (entry.length ?? 0)), 0)
  t.is(msg.varAbbrev, entry.var)

  // ECMWF .index: explicit lengths, MARS keys verbatim.
  const ecmwf = parseGribIndex(
    '{"domain": "g", "date": "20260610", "time": "0000", "step": "3", "levtype": "sfc", "param": "2t", "_offset": 0, "_length": 224}',
  )
  t.like(ecmwf[0], { var: '2t', offset: 0, length: 224, forecastTime: '3' })
  t.is(ecmwf[0].keys.levtype, 'sfc')
})

test('GribMessageFactory throws for unknown key', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const factory = GribMessageFactory.fromBuffer(data)

  t.throws(() => factory.getMessage('nonexistent_key_xyz'), {
    message: /not found/,
  })
})
