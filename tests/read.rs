extern crate gribberish;

use gribberish::message::{Message, read_messages};
use std::fs::File;
use std::io::Read;
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
        //assert_eq!(message.sections.len(), 8);

        let Ok(key) = message.key() else {
            println!("failed to get key");
            continue;
        };

        if keys.contains(&key) {
            dups.push(key);
        } else {
            keys.push(key);
        }

        let Ok(_data_point_count) = message.data_point_count() else {
            continue;
        };

        let Ok(_data) = message.data() else {
            continue;
        };

        // assert_eq!(data.len(), data_point_count);
    }

    println!("{key_count} keys, {dups} duplicates", key_count=keys.len(), dups=dups.len());
    keys.iter().for_each(|k: &String| println!("{k}"));
}
