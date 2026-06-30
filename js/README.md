# @mattnucc/gribberish

![CI](https://github.com/mpiannucci/gribberish/actions/workflows/js.yml/badge.svg)

Node.js bindings for [gribberish](https://crates.io/crates/gribberish), a Rust GRIB2 reader.

This package is a **reader only**. Loading GRIB2 bytes from local files, object stores, or network sources is intentionally left to your application.

## Install

```bash
yarn add @mattnucc/gribberish
```

or

```bash
npm install @mattnucc/gribberish
```

## Usage

Read the first message in a GRIB2 file:

```ts
import { readFileSync } from 'node:fs'
import { GribMessageFactory } from '@mattnucc/gribberish'

const data = readFileSync('sample.grib2')
const factory = GribMessageFactory.fromBuffer(data)
const first = factory.getMessage(factory.availableMessages[0])

console.log(first.varName, first.gridShape.rows, first.gridShape.cols)
```

Common entry points:

- `parseMessagesFromBuffer(buffer)` to parse all messages.
- `GribMessage.parseFromBuffer(buffer, offset)` to parse one message at a known offset.
- `GribMessageFactory.fromBuffer` to scan and fetch messages by key.
- `GribMessageMetadataFactory.fromBuffer` for metadata-first access.
- `parseGribIndex(text, fileSize?)` to parse a sidecar index file.

## Sidecar index files (`.idx` / `.index`)

Most NOAA models publish a wgrib2-style `.idx` text file next to every GRIB2
file, and ECMWF open data publishes a JSON-lines `.index` file. Both list each
message's byte offset, so a single field can be fetched with an HTTP Range
request instead of downloading the whole multi-hundred-MB file — including
from the browser:

```ts
import { GribMessage, parseGribIndex } from '@mattnucc/gribberish'

const url = 'https://noaa-gfs-bdp-pds.s3.amazonaws.com/gfs.20260612/00/atmos/gfs.t00z.pgrb2.0p25.f012'

// 1. fetch the index (~100 KB) and locate the message you want
const entries = parseGribIndex(await (await fetch(`${url}.idx`)).text())
const entry = entries.find((e) => e.var === 'TMP' && e.level === '2 m above ground')!

// 2. fetch just that message by byte range and parse it standalone
const res = await fetch(url, {
  headers: { Range: `bytes=${entry.offset}-${entry.offset + entry.length! - 1}` },
})
const msg = GribMessage.parseFromBuffer(new Uint8Array(await res.arrayBuffer()), 0)

console.log(msg.varName, msg.gridShape, msg.data.length)
```

The last entry of a NOAA index has no `length` unless you pass the GRIB file
size as the second argument (an open-ended `Range: bytes=N-` request also
works). ECMWF entries always carry explicit lengths, plus their MARS metadata
verbatim under `keys`. Note that cfgrib's pickled `.idx` cache files are an
unrelated format and not supported.

## WebAssembly targets

WebAssembly packages are optional, so install with a wasm target selector instead of adding the package explicitly.

- npm (v10.2+):

```bash
npm install --cpu=wasm32 @mattnucc/gribberish
```

- yarn v4:

```yaml
# .yarnrc.yml
supportedArchitectures:
  cpu:
    - current
    - wasm32
```

Then run `yarn install`.

In Node, set `NAPI_RS_FORCE_WASI=1` (or `NAPI_RS_FORCE_WASI=error`) if you want to force the WASM runtime path.

## Build and test

- `yarn install`
- `yarn build`
- `yarn test`

## Requirements

- Rust toolchain
- Node.js (CI validates on 20/22; release jobs run on 24)
- Yarn 4.x (`packageManager: yarn@4.12.0`)

## Release workflow

The `js` workflow publishes artifacts from GitHub Actions when you push a version tag.

1. `npm version [major | minor | patch | ...]`
2. `git push --follow-tags`

Make sure `NPM_TOKEN` is set in repository secrets.

> Do not run `npm publish` manually.
