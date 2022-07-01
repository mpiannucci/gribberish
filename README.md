# gribberish

Read [GRIB 2](https://en.wikipedia.org/wiki/GRIB) files with Rust.

See [`python`](python/README.md) for usage with `python`

See [`node`](node/README.md) for usage with `nodejs`

## Getting Started

Add the package in `Cargo.toml` unser `[dependencies]`:

```toml
gribberish = { git = "https://github.com/mpiannucci/gribberish" }
```

See [read.rs](tests/read.rs) for example usage for simple reading, or [message-dump](examples/message-dump/main.rs) for an example of dumping grib metadata to stdout. 

## License

[MIT](LICENSE) -  2022 Matthew Iannucci