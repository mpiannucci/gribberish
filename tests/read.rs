extern crate grib;

use grib::message::Message;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::vec::Vec;

#[test]
fn read_multi() {
    let multi_grib_path = Path::new("tests/data/multi_1.at_10m.t00z.f005.grib2");
    let mut multi_grib_file = File::open(multi_grib_path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    multi_grib_file.read_to_end(&mut raw_grib_data).expect("failed to read raw grib2 data");

    let grib_data = raw_grib_data.as_slice();
    let messages = Message::parse_all(raw_grib_data.as_slice());
    
    assert!(messages.len() > 0);
}
