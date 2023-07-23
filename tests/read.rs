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

#[test]
fn read_multi() {
    let grib_data = read_grib_messages("tests/data/multi_1.at_10m.t12z.f147.grib2");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    assert_ne!(messages.len(), 0);

    let mut keys = Vec::new();
    let mut dups = Vec::new();

    for message in messages {

        let key = message.key();
        assert!(key.is_ok());
        let key = key.unwrap();

        if keys.contains(&key) {
            dups.push(key);
        } else {
            keys.push(key);
        }

        let data = message.data();
        assert!(data.is_ok());
    }
}

#[test]
fn read_hrrr_ugrd_surface() {
    let read_data = read_grib_messages("tests/data/hrrr.t06z.wrfsfcf01-UGRD.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop();
    assert!(message.is_some());
    let message = message.unwrap();

    let start = Instant::now();
    let data = message.data();
    let end = Instant::now();
    println!("data() took {:?}", end.duration_since(start));

    assert!(data.is_ok());
}
