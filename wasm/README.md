# gribberish-wasm 

## WIP: Not working or functional at all currently due to c interfacing issues with wasm32-unknown-unknown target 

JavaScript bindings to `gribberish`

## Dependencies

Install `clang-8`

```bash
sudo apt install clang-8
```

Add the `wasm32-wasi` target

```
rustup target add wasm32-wasi
```

then build: 

```
cargo build --target wasm32-wasi
```

#### `wasm-pack` (not working yet)

First, install [`wasm-pack`](https://rustwasm.github.io/wasm-pack)


## Building

```bash
CC=clang-8 wasm-pack build

# add --target nodejs for node
# add --target web for web
# add --target bundler for webpack etc

```

## Publishing

```bash
wasm-pack publish
```
