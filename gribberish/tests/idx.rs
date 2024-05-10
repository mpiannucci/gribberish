use std::{fs::File, io::Read};
use std::fs::read_to_string;

use gribberish::message_metadata::scan_message_metadata;

extern crate gribberish;

pub fn read_grib_messages(path: &str) -> Vec<u8> {
    let mut grib_file = File::open(path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file.read_to_end(&mut raw_grib_data).expect("failed to read raw grib2 data");

    raw_grib_data
}

pub fn read_idx(path: &str) -> Vec<String> {
    read_to_string(path).unwrap().lines().map(|l| l.to_string()).collect()
}

#[test]
fn test_gfs_wave_idx_generation() {
    let read_data = read_grib_messages("tests/data/gfswave.t18z.atlocn.0p16.f001.grib2");
    let metadata = scan_message_metadata(read_data.as_slice());
    let mut idxs = metadata.iter()
        .map(|m| (m.1.0, m.1.2.as_idx(m.1.0)))
        .collect::<Vec<_>>();
    idxs.sort_by(|a, b| a.0.cmp(&b.0));

    let idx_lines = read_idx("tests/data/gfswave.t18z.atlocn.0p16.f001.grib2.idx");

    for (idx, line) in idx_lines.iter().enumerate() {
        assert_eq!(line, &idxs[idx].1);
    }
}