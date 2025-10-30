//! Simple GRIB2 Reader Example
//!
//! A minimal example showing the basics of reading GRIB2 files.
//!
//! Usage:
//!   cargo run --example simple_reader

use gribberish::api::read_all_messages;
use gribberish::backends::BackendType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Select backend (native Rust or eccodes)
    let backend = BackendType::Native;

    // 2. Read the GRIB2 file
    let file_path = "/Users/maxwellgrover/projects/grib-reading/data/gfs.t18z.pgrb2.0p25.f186-RH.grib2";
    let data = std::fs::read(file_path)?;

    // 3. Parse all messages
    let messages = read_all_messages(&data, backend)?;

    // 4. Display results
    println!("Found {} message(s) in file\n", messages.len());

    for (i, msg) in messages.iter().enumerate() {
        println!("Message {}:", i + 1);
        println!("  Variable: {} ({})", msg.metadata.var, msg.metadata.name);
        println!("  Units: {}", msg.metadata.units);
        println!("  Grid: {} x {}", msg.metadata.grid_shape.0, msg.metadata.grid_shape.1);
        println!("  Data points: {}", msg.data.len());
        println!("  Date: {}", msg.metadata.forecast_date);

        if !msg.data.is_empty() {
            let valid: Vec<f64> = msg.data.iter().filter(|v| v.is_finite()).copied().collect();
            if !valid.is_empty() {
                let min = valid.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let avg = valid.iter().sum::<f64>() / valid.len() as f64;
                println!("  Range: {:.2} to {:.2} (avg: {:.2})", min, max, avg);
            }
        }
        println!();
    }

    Ok(())
}
