# Rust API Reference

The core gribberish library is written in Rust for maximum performance.

:::{card} View Full Documentation
:link: ../api/rust/gribberish/index.html
:link-type: url
:text-align: center

**📚 Rust API Documentation →**
^^^
Complete rustdoc-generated API reference
:::

## Installation

Add to your `Cargo.toml`:
```toml
[dependencies]
gribberish = "0.3"
```

## Quick Example
```rust
use gribberish::{Grib2, Message};
use std::fs::File;
use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::open("data.grib2")?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    let grib = Grib2::from_bytes(&buffer)?;
    
    for message in grib.messages() {
        println!("Parameter: {:?}", message.parameter());
        println!("Level: {:?}", message.level());
    }
    
    Ok(())
}
```

## Crate Structure

::::{grid} 1 2 2 3
:gutter: 2

:::{grid-item-card} gribberish
Core GRIB2 parsing and decoding
:::

:::{grid-item-card} gribberish-types
Common type definitions and traits
:::

:::{grid-item-card} gribberish-macros
Procedural macros for code generation
:::

::::

## Performance Tips

- Use `Grib2::from_reader` for streaming large files
- Enable the `parallel` feature for multi-threaded decoding
- Consider memory-mapping files with `memmap2` for repeated access