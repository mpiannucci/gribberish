extern crate chrono;
extern crate grib;

use chrono::prelude::*;
use std::error::Error;
use grib::message::Message;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use std::env;
use std::process;

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

    println!("Hello, world!");
}
