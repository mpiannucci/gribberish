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
