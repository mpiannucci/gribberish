extern crate chrono;
extern crate gribberish;

use chrono::prelude::*;
use std::error::Error;
use gribberish::message::Message;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use std::env;
use std::process;
use std::fmt::Display;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("You must specify a grib2 file to read");
        process::exit(0);
    }

    let grib_path = &args[1];
    let mut grib_file = File::open(grib_path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file.read_to_end(&mut raw_grib_data).expect("failed to read raw grib2 data");

    let messages = Message::parse_all(raw_grib_data.as_slice());

    println!("GRIB2 file read: {}", grib_path);
    println!("Message count: {}", messages.len());
    println!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", 
        "Message #", 
        "Variable", 
        "Units", 
        "Date", 
        "Region", 
        "Grid Resolution",
        "Grid",
        "Data Template Id", 
        "Data Point Count");
    println!("------------------------------------------------------------------------------------------------------------");

    messages.iter().enumerate().for_each(|m| {
        println!("Discipline {}", m.1.discipline().unwrap());
        match m.1.metadata() {
            Ok(metadata) => {
                println!("{}\t{}\t{}\t{}\t{:?}\t{:?}\t{:?}\t{}\t{}", 
                m.0, 
                metadata.variable_abbreviation, 
                metadata.units, 
                metadata.forecast_date, 
                metadata.region, 
                metadata.location_resolution,
                metadata.location_grid,
                metadata.data_template_number, 
                metadata.data_point_count);
            }, 
            Err(e) => {
                println!("{:?}", m.1.location_region().unwrap());
                println!("{:?}", m.1.location_grid_dimensions().unwrap());
                println!("Failed to read message metadata: {}", e)
            }
        }
    });
}
