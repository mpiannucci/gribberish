extern crate chrono;
extern crate grib;
extern crate futures;
extern crate tokio;
extern crate reqwest;
extern crate bytes;
extern crate csv;
 
use chrono::prelude::*;
use std::error::Error;
use grib::message::Message;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;
use std::collections::HashMap;

fn read_grib_messages(path: &str) -> Vec<u8> {
    let mut grib_file = File::open(path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file.read_to_end(&mut raw_grib_data).expect("failed to read raw grib2 data");

    raw_grib_data
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // from https://nomads.ncep.noaa.gov/cgi-bin/filter_wave.pl?file=multi_1.nww3.t12z.grib2&subregion=&leftlon=288&rightlon=289&toplat=41.5&bottomlat=40.5&dir=%2Fmulti_1.20201209
    let grib_data = read_grib_messages("./grib/examples/generate_forecast/multi_1.nww3.t12z.grib2");
    let messages = Message::parse_all(grib_data.as_slice());

    let mut data_map: HashMap<DateTime<Utc>, Vec<(String, f64)>> = HashMap::new();

    println!("{}", messages.len());
    for message in messages {
        let metadata = message.metadata();
        if let Err(_) = metadata {
            continue;
        }
        let metadata = metadata.unwrap();
        
        let datapoints = message.data();
        if let Err(e) = datapoints {
            println!("{}: {}", metadata.variable_abbreviation, e);
            continue;
        }
        let datapoints = datapoints.unwrap();

        if !data_map.contains_key(&metadata.forecast_date) {
            data_map.insert(metadata.forecast_date, Vec::new());
        }

        if let Some(data_collection) = data_map.get_mut(&metadata.forecast_date) {
            data_collection.push((metadata.variable_abbreviation, datapoints.first().unwrap().clone()));
        }
    }

    println!("{:?}", data_map.keys());

    Ok(())
}