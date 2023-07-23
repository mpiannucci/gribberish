extern crate gribberish;

use gribberish::message::{Message, read_messages};
use std::fs::File;
use std::io::Read;
use std::time::Instant;
use std::vec::Vec;

fn read_grib_messages(path: &str) -> Vec<u8> {
    let mut grib_file = File::open(path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file.read_to_end(&mut raw_grib_data).expect("failed to read raw grib2 data");

    raw_grib_data
}

// #[test]
// fn read_jpeg() {
//     let grib_data = read_grib_messages("tests/data/multi_1.at_10m.t12z.f147.grib2");
//     let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

//     let message_count = messages.len();
//     assert_ne!(message_count, 0);

//     let mut keys = Vec::new();
//     let mut dups = Vec::new();

//     let mut durations = Vec::new();

//     for message in messages {

//         let key = message.key();
//         assert!(key.is_ok());
//         let key = key.unwrap();

//         if keys.contains(&key) {
//             dups.push(key);
//         } else {
//             keys.push(key);
//         }

//         let start = Instant::now();
//         let data = message.data();
//         let end = Instant::now();
//         assert!(data.is_ok());
//         durations.push(end.duration_since(start));
//     }

//     let duration_sum: u128 = durations
//         .iter()
//         .map(|d| d.as_millis())
//         .sum();
//     let duration_mean = duration_sum / message_count as u128;
//     println!("jpeg unpacking data() took an average of {duration_mean}ms per message");
// }

// #[test]
// fn read_simple() {
//     let read_data = read_grib_messages("tests/data/hrrr.t06z.wrfsfcf01-UGRD.grib2");
//     let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
//     assert_eq!(messages.len(), 1);

//     let message = messages.pop();
//     assert!(message.is_some());
//     let message = message.unwrap();

//     let start = Instant::now();
//     let data = message.data();
//     let end = Instant::now();
//     assert!(data.is_ok());
//     let data = data.unwrap();
//     println!("simple unpacking data() took {:?} for {} data points", end.duration_since(start), data.len());
//     assert!((data[1000] - -4.46501350402832).abs() < 0.0000001);
// }

#[test]
fn read_spatial_differenced_complex() {
    let read_data = read_grib_messages("tests/data/hrrr.t06z.wrfsfcf01-TMP.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop();
    assert!(message.is_some());
    let message = message.unwrap();

    let start = Instant::now();
    let data = message.data();
    let end = Instant::now();
    assert!(data.is_ok());
    let data = data.unwrap();
    println!("spatial complex unpacking data() took {:?} for {} data points", end.duration_since(start), data.len());
    assert!((data[1000] - 303.7372741699219).abs() < 0.0000001);
}