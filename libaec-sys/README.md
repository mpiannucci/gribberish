# libaec-sys

Raw FFI bindings to the [libaec](https://gitlab.dkrz.de/k202009/libaec) (Adaptive Entropy Coding) library.

## Overview

This crate provides low-level Rust bindings to the libaec C library, which implements CCSDS lossless compression as used in GRIB2 meteorological files.

## Build Process

When you compile this crate, it will:

1. **Download** libaec v1.1.4 source code from the official GitLab repository
2. **Compile** libaec as a static library using CMake
3. **Generate** Rust FFI bindings using bindgen
4. **Link** the static library into your Rust binary

## Dependencies

Build dependencies:
- Git (to download source)
- CMake (to build libaec)
- A C compiler (for libaec compilation)
- Clang (for bindgen header parsing)

Runtime dependencies: None (statically linked)

## Generated Bindings

The build process automatically generates Rust bindings for:

- **Constants**: `AEC_OK`, `AEC_DATA_MSB`, `AEC_DATA_SIGNED`, etc.
- **Structures**: `aec_stream` with proper memory layout
- **Functions**: `aec_decode_init()`, `aec_decode()`, `aec_decode_end()`, etc.

## Usage

This is a low-level `-sys` crate. Most users should use the higher-level `libaec` wrapper crate instead, which provides safe Rust APIs.

If you need to use this crate directly:

```rust
use libaec_sys::*;

unsafe {
    let mut stream: aec_stream = std::mem::zeroed();
    // Configure stream fields...
    let ret = aec_decode_init(&mut stream);
    // Handle return code...
}
```
