extern crate bytes;
extern crate chrono;
extern crate csv;
extern crate futures;
extern crate grib;
extern crate reqwest;
extern crate tokio;

use bytes::Bytes;
use chrono::prelude::*;
use futures::{stream, StreamExt};
use grib::message::Message;
use reqwest::Url;
use tokio::time::Instant;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;

fn read_grib_messages(path: &str) -> Vec<u8> {
    let mut grib_file = File::open(path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file
        .read_to_end(&mut raw_grib_data)
        .expect("failed to read raw grib2 data");

    raw_grib_data
}

fn generate_grib_url(date: &DateTime<Utc>, valid_hour: i32) -> Url {
    let raw_url = format!("https://ftp.ncep.noaa.gov/data/nccf/com/gfs/prod/gfs.{}/{:02}/wave/gridded/gfswave.t{:02}z.atlocn.0p16.f{:03}.grib2", 
    date.format("%Y%m%d"), 
    date.hour(), 
    date.hour(), 
    valid_hour);

    Url::parse(&raw_url).unwrap()
}

pub fn mean(data: &Vec<f64>) -> f64 {
    let filtered_data: Vec<_> = data
        .iter()
        .filter(|v| !v.is_nan())
        .collect();

    let count = filtered_data.len() as f64;
    filtered_data.into_iter().sum::<f64>() / count
}

pub fn latest_model_time() -> DateTime<Utc> {
    let now = Utc::now();
    match now.hour() {
        0..=5 => now.with_hour(18).unwrap().with_day(now.day() - 1).unwrap(),
        6..=11 => now.with_hour(0).unwrap(),
        12..=17 => now.with_hour(6).unwrap(), 
        _ => now.with_hour(12).unwrap(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let location = (41.0, 288.5);

    let model_time = latest_model_time();
    let urls = (0..61).collect::<Vec<i32>>().iter().map(|i| {
        generate_grib_url(&model_time, i * 3)
    }).collect::<Vec<Url>>();

    // Download the data from NOAA's grib endpoint
    let results: Vec<Option<Bytes>> = stream::iter(urls.into_iter().map(|url| async move {
        let rurl = Url::parse(url.as_str()).unwrap();
        match reqwest::get(rurl).await {
            Ok(resp) => match resp.bytes().await {
                Ok(b) => Some(b),
                Err(_) => None,
            },
            Err(_) => None,
        }
    }))
    .buffered(8)
    .collect()
    .await;

    println!("Downloaded Data: {:?}", start.elapsed());
    
    // Parse out the data into data and metadata
    let all_grib_data: Vec<_> = results
        .into_iter()
        .filter_map(|b| {
            match b {
                Some(b) => {
                    let data: Vec<_> = grib::message::Message::parse_all(b.clone().as_ref())
                    .iter()
                    .filter(|m| m.metadata().is_ok())    
                    .map(|m| (m.metadata().unwrap(), m.data_at_location(&location)))
                    .collect();
                    Some(data)
                },
                None => None,
            }
        }).collect();

    println!("Parsed Model Data: {:?}", start.elapsed());

    let mut wtr = csv::Writer::from_path("./ri_wave_data.csv")?;

    // Collect the variables and write out the result as the header
    let mut vars: Vec<_> = all_grib_data[0]
        .iter()
        .map(|m| format!("{} ({})", (m.0).variable_abbreviation.clone(), (m.0).units ))
        .collect();
    if vars.len() == 0 {
        return Err(Box::from("No variables read"));
    }
    vars.insert(0, String::from("TIME"));
    wtr.write_record(vars)?;

    // Then collect the mean of every value 
    all_grib_data.iter().for_each(|dt| {
        let mut point_data: Vec<_> = dt
            .iter()
            .map(|d| {
                let value = match &d.1 {
                    Ok(vals) => *vals,
                    Err(err) => {
                        println!("{}", err);
                        std::f64::NAN
                    }
                };
                format!("{:.2}", value)
            }).collect();
        if point_data.len() > 0 {
            point_data.insert(0, dt[0].0.forecast_date.to_rfc3339());
        }

        let _ = wtr.write_record(point_data);
    });

    wtr.flush()?;

    println!("Wrote Model Data: {:?}", start.elapsed());

    Ok(())
}
