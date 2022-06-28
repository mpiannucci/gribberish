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
    let messages = read_messages(grib_data).collect::<Vec<Message>>();

    assert_ne!(messages.len(), 0);

    for message in messages {
        //assert_eq!(message.sections.len(), 8);

        let field = message.data_point_count();
        if let Err(_) = field {
            continue;
        }

        let data = message.data();
        if let Err(_) = data {
            continue;
        }

        let data = data.unwrap();
        println!("Data: {:?}", data);
    }
}
