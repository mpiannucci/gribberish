//! Example: Reading GRIB files with the native Rust backend
//!
//! This example demonstrates how to read both GRIB1 and GRIB2 files
//! using the native Rust implementation.
//!
//! Usage:
//!   cargo run --example read_grib_data
//!   cargo run --example read_grib_data -- /path/to/file.grib2

use gribberish::api::{
    build_message_index, read_all_messages, read_message_at_offset, scan_messages,
};
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================================");
    println!("  GRIB Reader Example - Native Rust Backend");
    println!("  Supports both GRIB1 and GRIB2 formats");
    println!("=================================================\n");

    // Get file path from command line or use default
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        Path::new(&args[1]).to_path_buf()
    } else {
        // Default to the smaller test file
        Path::new("tests/data/gfs.t18z.pgrb2.0p25.f186-RH.grib2")
            .to_path_buf()
    };

    if !file_path.exists() {
        eprintln!("Error: File not found: {}", file_path.display());
        eprintln!("\nUsage:");
        eprintln!("  cargo run --example read_grib_data");
        eprintln!("  cargo run --example read_grib_data -- /path/to/file.grib2");
        return Ok(());
    }

    println!("Reading file: {}", file_path.display());
    let file_size = std::fs::metadata(&file_path)?.len();
    println!("File size: {} bytes ({:.2} MB)\n", file_size, file_size as f64 / 1_048_576.0);

    // For large files, just scan first, then selectively parse
    if file_size > 10_000_000 {
        // > 10 MB
        handle_large_file(&file_path)?;
    } else {
        handle_small_file(&file_path)?;
    }

    println!("\n=================================================");
    println!("Example completed successfully!");
    println!("=================================================");

    Ok(())
}

/// Handle small files - read everything
fn handle_small_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Reading All Messages ---\n");

    let data = std::fs::read(file_path)?;
    let messages = read_all_messages(&data)?;

    println!("Found {} message(s)\n", messages.len());

    for (i, msg) in messages.iter().enumerate() {
        println!("═══════════════════════════════════════════════");
        println!("Message {}/{}", i + 1, messages.len());
        println!("═══════════════════════════════════════════════");
        println!("Variable:           {}", msg.metadata.var);
        println!("Name:               {}", msg.metadata.name);
        println!("Units:              {}", msg.metadata.units);
        println!("Grid shape:         {} x {}", msg.metadata.grid_shape.0, msg.metadata.grid_shape.1);
        println!("Total values:       {}", msg.data.len());
        println!("Regular grid:       {}", msg.metadata.is_regular_grid);
        println!("Reference date:     {}", msg.metadata.reference_date);
        println!("Forecast date:      {}", msg.metadata.forecast_date);
        println!("Level type:         {:?}", msg.metadata.first_fixed_surface_type);

        if let Some(level) = msg.metadata.first_fixed_surface_value {
            println!("Level value:        {}", level);
        }

        // Print data statistics
        if !msg.data.is_empty() {
            print_data_stats(&msg.data);
        }

        println!();
    }

    Ok(())
}

/// Handle large files - scan and sample
fn handle_large_file(file_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Scanning Large File ---\n");

    let data = std::fs::read(file_path)?;

    // 1. Scan without parsing
    let offsets = scan_messages(&data);
    println!("Found {} messages by scanning\n", offsets.len());

    // 2. Build an index of variables
    println!("Building message index...");
    let index = build_message_index(&data)?;
    println!("Found {} unique variables:\n", index.len());

    for (var, locations) in &index {
        println!("  {} : {} message(s)", var, locations.len());
    }

    // 3. Read a few messages as samples
    println!("\n--- Sample Messages ---\n");
    let sample_count = 3.min(offsets.len());

    for i in 0..sample_count {
        let (offset, _) = offsets[i];
        let msg = read_message_at_offset(&data, offset)?;

        println!("═══════════════════════════════════════════════");
        println!("Sample Message {}/{}", i + 1, sample_count);
        println!("═══════════════════════════════════════════════");
        println!("Variable:      {}", msg.metadata.var);
        println!("Name:          {}", msg.metadata.name);
        println!("Grid:          {} x {}", msg.metadata.grid_shape.0, msg.metadata.grid_shape.1);
        println!("Date:          {}", msg.metadata.forecast_date);

        if !msg.data.is_empty() {
            print_data_stats(&msg.data);
        }

        println!();
    }

    Ok(())
}

/// Print statistics about the data values
fn print_data_stats(data: &[f64]) {
    let valid: Vec<f64> = data.iter().filter(|v| v.is_finite()).copied().collect();

    if valid.is_empty() {
        println!("Data:              All values are missing/invalid");
        return;
    }

    let min = valid.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = valid.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = valid.iter().sum();
    let mean = sum / valid.len() as f64;

    println!("Valid values:      {}", valid.len());
    println!("Missing values:    {}", data.len() - valid.len());
    println!("Min:               {:.2}", min);
    println!("Max:               {:.2}", max);
    println!("Mean:              {:.2}", mean);
}
