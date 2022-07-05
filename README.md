# gribberish

Read [GRIB 2](https://en.wikipedia.org/wiki/GRIB) files with Rust.

See [`python`](python/README.md) for usage with `python` and `xarray`

See [`node`](node/README.md) for usage with `nodejs`

## Getting Started

Add the package in `Cargo.toml` unser `[dependencies]`:

```toml
gribberish = { git = "https://github.com/mpiannucci/gribberish" }
```

See [read.rs](tests/read.rs) for example usage for simple reading, or [message-dump](examples/message-dump/main.rs) for an example of dumping grib metadata to stdout. 

## Building

Download the vendored `spec` dependencies: 

```bash
git submodule update --init
```

Then build normally

```bash
cargo build
```

## License

[MIT](LICENSE) -  2022 Matthew Iannucci