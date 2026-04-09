import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { join, dirname } from 'node:path'

import test from 'ava'

import { GribMessage, GribMessageFactory, GribMessageMetadataFactory, parseMessagesFromBuffer } from '../index'

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

test('GribMessageFactory throws for unknown key', (t) => {
  const data = readFileSync(join(DATA_DIR, 'hrrr.t06z.wrfsfcf01-TMP.grib2'))
  const factory = GribMessageFactory.fromBuffer(data)

  t.throws(() => factory.getMessage('nonexistent_key_xyz'), {
    message: /not found/,
  })
})
