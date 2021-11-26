extern crate chrono;
extern crate gribberish;

use chrono::prelude::*;
use gribberish::message::Message;
use gribberish::templates::product::ProductDiscipline;
use std::env;
use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::Read;
use std::process;
use std::vec::Vec;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("You must specify a grib2 file to read");
        process::exit(0);
    }

    let grib_path = &args[1];
    let mut grib_file = File::open(grib_path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file
        .read_to_end(&mut raw_grib_data)
        .expect("failed to read raw grib2 data");

    let messages = Message::parse_all(raw_grib_data.as_slice());

    println!("GRIB2 file read: {}", grib_path);
    println!("Message count: {}", messages.len());
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        "Message #",
        "Variable",
        "Units",
        "Date",
        "Region",
        "Grid",
        "Data Template Id",
        "Data Point Count"
    );
    println!("------------------------------------------------------------------------------------------------------------");

    messages.iter().enumerate().for_each(|m| {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            m.0,
            match m.1.parameter() { Ok(p) => p.abbrev, Err(_) => "--".into()},
            match m.1.parameter() {Ok(p) => p.unit, Err(_) => "--".into()},
            match m.1.forecast_date() { Ok(d) => format!("{}", d), Err(_) => "--".into()},
            match m.1.location_region() {Ok(r) => format!("{:?}", r), Err(_) => "--".into()},
            match m.1.location_grid_dimensions() {Ok(r) => format!("{:?}", r), Err(_) => "--".into()},
            match m.1.data_template_number() {Ok(t) => format!("{}", t), Err(_) => "--".into()},
            match m.1.data_point_count() {Ok(c) => format!("{}", c), Err(_) => "--".into()},
        );
    });
}
