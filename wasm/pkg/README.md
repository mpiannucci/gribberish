# gribberish-wasm 

JavaScript bindings to `gribberish`

## Dependencies

First, install [`wasm-pack`](https://rustwasm.github.io/wasm-pack)

Then, install `clang-8`

```bash
sudo apt install clang-8
```

## Building

```bash
CC=clang-8 wasm-pack build
```

## Publishing

```bash
wasm-pack publish
```
