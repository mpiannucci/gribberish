# Examples

## Message Dump

Reads every grib message in the specified grib2 file and dumps the metadata for each message to the console 

```bash
cargo run --release --example message_dump -- /path/to/grib.grib2
```

## Split Messages

Scans a grib2 file and writes each message to a separate grib2 file. optionally specifcy the output folder if desired to be different from the input file's parent folder./

```bash
cargo run --release --example split_messages -- -o /path/to/dest /path/to/grib.grib2
```