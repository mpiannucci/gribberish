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
    let grib_data = read_grib_messages("./grib/examples/generate_forecast/nww3.t06z.grib.grib2");
    let messages = Message::parse_all(grib_data.as_slice());

    let mut data_map: HashMap<DateTime<Utc>, Vec<(String, f64)>> = HashMap::new();

    const FORECAST_LOCATION: (f64, f64) = (40.969, 288.873);

    for message in messages {
        let metadata = message.metadata();
        if let Err(_) = metadata {
            continue;
        }
        let metadata = metadata.unwrap();
        
        let datapoint = message.data_at_location(&FORECAST_LOCATION);
        if let Err(e) = datapoint {
            println!("{}", e);
            continue;
        }
        let datapoint = datapoint.unwrap();

        if let Some(data_collection) = data_map.get_mut(&metadata.forecast_date) {
            data_collection.push((metadata.variable_abbreviation, datapoint));
        }
    }

    println!("{:?}", data_map.keys());

    Ok(())
}