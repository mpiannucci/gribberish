extern crate chrono;
extern crate gribberish;

use gribberish::message::{MessageIterator};
use std::env;
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

    let message_iter = MessageIterator::from_data(raw_grib_data.as_slice(), 0);

    println!("GRIB2 file read: {}", grib_path);
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        "Message #",
        "Parameter Index",
        "Variable",
        "Name",
        "Units",
        "Generating Process",
        "Statistical Process",
        "Fixed Surface",
        "Date",
        "Product Template Id",
        "Grid Template Id",
        "BBOX",
        "Grid",
        "Data Template Id",
        "Data Point Count",
    );
    println!("------------------------------------------------------------------------------------------------------------");

    message_iter.enumerate().for_each(|(idx, m)| {
        let bbox = m.latlng_projector().expect("Failed to get message projection").bbox();

        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            idx,
            match m.parameter_index() {
                Ok(p) => p,
                Err(_) => "--".into(),
            },
            match m.variable_abbrev() {
                Ok(p) => p,
                Err(_) => "--".into(),
            },
            match m.variable_name() {
                Ok(p) => p,
                Err(_) => "--".into(),
            },
            match m.unit() {
                Ok(p) => p,
                Err(_) => "--".into(),
            },
            match m.generating_process() {
                Ok(g) => format!("{g}"),
                Err(_) => "--".into(),
            },
            match (m.statistical_process_type(), m.derived_forecast_type()) {
                (Ok(Some(s)), Ok(Some(d))) => format!("{s:?} {d:?}"),
                (Ok(Some(s)), Ok(None)) => format!("{s:?}"),
                (Ok(None), Ok(Some(d))) => format!("{d:?}"),
                _ => "--".into(),
            },
            match m.first_fixed_surface() {
                Ok(f) => format!("{} {}", f.0, f.1.unwrap_or(0.0)),
                Err(_) => "--".into(),
            },
            match (m.time_interval_end(), m.forecast_date()) {
                (Ok(None), Ok(d)) => format!("{d}"),
                (Ok(Some(d)), _) => format!("{d}"),
                _ => "--".into(),
            },
            match m.product_template_id() {
                Ok(p) => format!("{p}"),
                Err(_) => "--".into(),
            },
            match m.grid_template_id() {
                Ok(d) => format!("{d}"),
                Err(_) => "--".into(),
            },
            format!("{:?}", bbox),
            match m.grid_dimensions() {
                Ok(r) => format!("{:?}", r),
                Err(_) => "--".into(),
            },
            match m.data_template_number() {
                Ok(t) => format!("{t}"),
                Err(_) => "--".into(),
            },
            match m.data_point_count() {
                Ok(c) => format!("{c}"),
                Err(_) => "--".into(),
            },
        );
    });
}
