# gribberish

Read [GRIB 2](https://en.wikipedia.org/wiki/GRIB) files with Rust.

## Getting Started

Add the package in `Cargo.toml` to `[dependencies]`:

```toml
gribberish = { git = "https://github.com/mpiannucci/gribberish" }
```

The following `features` are available:

`png`: Allows unpacking PNG encoded data messages

`jpeg`: Allows unpacking JPEG2000 encoded data messages

By default, both `png` and `jpeg` are enabled.

See [read.rs](tests/read.rs) for example usage for simple reading, or [message-dump](examples/message-dump/main.rs) for an example of dumping grib metadata to stdout.

## License

[MIT](LICENSE) -  2023 Matthew Iannucci
