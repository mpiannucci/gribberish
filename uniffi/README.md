# gribberish-uniffi

Universal bindings via [Mozilla UniFFI](https://github.com/mozilla/uniffi-rs)

```
cargo install uniffi_bindgen
```

## Build the library and binding code

```
cargo build
```

## Generate the bindings 

```
# Swift
uniffi-bindgen generate src/gribberish.udl --language swift

# Kotlin
uniffi-bindgen generate src/gribberish.udl --language kotlin
```