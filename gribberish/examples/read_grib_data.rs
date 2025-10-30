//! Example: Reading GRIB2 files with the new backend framework
//!
//! This example demonstrates how to use the revised gribberish framework
//! to read real GRIB2 files.
//!
//! Usage:
//!   cargo run --example read_grib_data
//!   cargo run --example read_grib_data --features gribberish/eccodes
//!   cargo run --example read_grib_data -- /path/to/file.grib2

use gribberish::api::{
    build_message_index, read_all_messages, read_message_at_offset, scan_messages_with_backend,
};
use gribberish::backends::BackendType;
use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=================================================");
    println!("  GRIB2 Reader Example - Revised Framework");
    println!("=================================================\n");

    // Determine which backend to use
    #[cfg(feature = "eccodes")]
    let backend = {
        println!("✓ Using eccodes backend\n");
        BackendType::Eccodes
    };

    #[cfg(not(feature = "eccodes"))]
    let backend = {
        println!("✓ Using native Rust backend\n");
        BackendType::Native
    };

    // Get file path from command line or use default
    let args: Vec<String> = env::args().collect();
    let file_path = if args.len() > 1 {
        Path::new(&args[1]).to_path_buf()
    } else {
        // Default to the smaller test file
        Path::new("/Users/maxwellgrover/projects/grib-reading/data/gfs.t18z.pgrb2.0p25.f186-RH.grib2")
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
        handle_large_file(&file_path, &backend)?;
    } else {
        handle_small_file(&file_path, &backend)?;
    }

    println!("\n=================================================");
    println!("Example completed successfully!");
    println!("=================================================");

    Ok(())
}

/// Handle small files - read everything
fn handle_small_file(
    file_path: &Path,
    backend: &BackendType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Reading All Messages ---\n");

    let data = std::fs::read(file_path)?;
    let messages = read_all_messages(&data, *backend)?;

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

        println!("Discipline:         {}", msg.metadata.discipline);
        println!("Category:           {}", msg.metadata.category);
        println!("Data compression:   {}", msg.metadata.data_compression);
        println!("Has bitmap:         {}", msg.metadata.has_bitmap);
        println!("Projection:         {}", msg.metadata.proj);

        // Show data statistics
        if !msg.data.is_empty() {
            let valid_data: Vec<f64> = msg
                .data
                .iter()
                .filter(|v| v.is_finite())
                .copied()
                .collect();

            if !valid_data.is_empty() {
                let min = valid_data.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = valid_data
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max);
                let sum: f64 = valid_data.iter().sum();
                let avg = sum / valid_data.len() as f64;

                println!("\nData Statistics:");
                println!("  Valid values:     {}/{}", valid_data.len(), msg.data.len());
                println!("  Min:              {:.4}", min);
                println!("  Max:              {:.4}", max);
                println!("  Average:          {:.4}", avg);

                // Show first few values
                println!("  First 10 values:");
                for (i, val) in msg.data.iter().take(10).enumerate() {
                    println!("    [{}] = {:.4}", i, val);
                }
            }
        }
        println!();
    }

    Ok(())
}

/// Handle large files - scan and sample
fn handle_large_file(
    file_path: &Path,
    backend: &BackendType,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Large File Detected - Scanning Only ---\n");

    let data = std::fs::read(file_path)?;

    println!("Step 1: Scanning for message locations...");
    let scanned = scan_messages_with_backend(&data, *backend);
    println!("Found {} message(s)\n", scanned.len());

    if scanned.is_empty() {
        println!("No GRIB messages found in file.");
        return Ok(());
    }

    println!("Message offsets:");
    for (i, (offset, size)) in scanned.iter().enumerate().take(20) {
        println!(
            "  Message {:4}: offset={:10}, size={:8} bytes",
            i + 1,
            offset,
            size
        );
    }
    if scanned.len() > 20 {
        println!("  ... and {} more messages", scanned.len() - 20);
    }
    println!();

    println!("Step 2: Building variable index...");
    match build_message_index(&data, *backend) {
        Ok(index) => {
            println!("Found {} unique variable(s):\n", index.len());

            for (var, locations) in &index {
                println!("  {:10} : {} message(s)", var, locations.len());
            }
            println!();
        }
        Err(e) => {
            println!("⚠️  Unable to build complete index: {}", e);
            println!("    (Some messages may use unsupported product templates)\n");
            println!("    💡 Tip: Try using --features gribberish/eccodes for full support\n");
        }
    }

    println!("Step 3: Reading first message as sample...");
    let first_offset = scanned[0].0;
    let first_msg = match read_message_at_offset(&data, first_offset, *backend) {
        Ok(msg) => msg,
        Err(e) => {
            println!("⚠️  Unable to parse first message: {}", e);
            println!("    Scanned {} messages successfully.", scanned.len());
            return Ok(());
        }
    };

    println!("═══════════════════════════════════════════════");
    println!("First Message Details");
    println!("═══════════════════════════════════════════════");
    println!("Variable:           {}", first_msg.metadata.var);
    println!("Name:               {}", first_msg.metadata.name);
    println!("Units:              {}", first_msg.metadata.units);
    println!("Grid shape:         {} x {}", first_msg.metadata.grid_shape.0, first_msg.metadata.grid_shape.1);
    println!("Total values:       {}", first_msg.data.len());
    println!("Reference date:     {}", first_msg.metadata.reference_date);
    println!("Forecast date:      {}", first_msg.metadata.forecast_date);

    if !first_msg.data.is_empty() {
        let valid_data: Vec<f64> = first_msg
            .data
            .iter()
            .filter(|v| v.is_finite())
            .copied()
            .collect();

        if !valid_data.is_empty() {
            let min = valid_data.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = valid_data
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max);
            let sum: f64 = valid_data.iter().sum();
            let avg = sum / valid_data.len() as f64;

            println!("\nData Statistics:");
            println!("  Valid values:     {}/{}", valid_data.len(), first_msg.data.len());
            println!("  Min:              {:.4}", min);
            println!("  Max:              {:.4}", max);
            println!("  Average:          {:.4}", avg);
        }
    }
    println!();

    println!("💡 Tip: For large files, use the index to selectively read only the variables you need.");

    Ok(())
}
