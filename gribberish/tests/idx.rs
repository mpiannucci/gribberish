use std::fs::read_to_string;
use std::{fs::File, io::Read};

use chrono::{TimeZone, Utc};
use gribberish::index::parse_index;
use gribberish::message_metadata::scan_message_metadata;

extern crate gribberish;

pub fn read_grib_messages(path: &str) -> Vec<u8> {
    let mut grib_file = File::open(path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file
        .read_to_end(&mut raw_grib_data)
        .expect("failed to read raw grib2 data");

    raw_grib_data
}

pub fn read_idx(path: &str) -> Vec<String> {
    read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| l.to_string())
        .collect()
}

#[test]
fn test_gfs_wave_idx_generation() {
    let read_data = read_grib_messages("../test-data/gfswave.t18z.atlocn.0p16.f001.grib2");
    let metadata = scan_message_metadata(read_data.as_slice());
    let mut idxs = metadata
        .iter()
        .map(|m| (m.1 .0, m.1 .2.as_idx(m.1 .0)))
        .collect::<Vec<_>>();
    idxs.sort_by(|a, b| a.0.cmp(&b.0));

    let idx_lines = read_idx("../test-data/gfswave.t18z.atlocn.0p16.f001.grib2.idx");

    for (idx, line) in idx_lines.iter().enumerate() {
        assert_eq!(line, &idxs[idx].1);
    }
}

#[test]
fn test_index_parsing_locates_messages() {
    // NOAA .idx: parsed entries resolve to exactly the byte ranges and
    // variables a full scan of the GRIB file finds.
    let data = read_grib_messages("../test-data/gfswave.t18z.atlocn.0p16.f001.grib2");
    let idx_text = read_to_string("../test-data/gfswave.t18z.atlocn.0p16.f001.grib2.idx").unwrap();
    let entries = parse_index(&idx_text, Some(data.len() as u64)).unwrap();

    let mut scanned: Vec<_> = scan_message_metadata(&data).into_values().collect();
    scanned.sort_by_key(|(index, _, _)| *index);

    assert_eq!(entries.len(), scanned.len());
    for (entry, (_, offset, meta)) in entries.iter().zip(scanned.iter()) {
        assert_eq!(entry.offset, *offset as u64);
        assert_eq!(entry.length, Some(meta.message_size as u64));
        assert_eq!(entry.var.as_deref(), Some(meta.var.as_str()));
        assert_eq!(entry.reference_date, Some(meta.reference_date));
    }

    // ECMWF .index: explicit offsets and lengths, MARS keys kept verbatim.
    let text = concat!(
        r#"{"domain": "g", "date": "20260610", "time": "0000", "step": "3", "levtype": "sfc", "param": "2t", "_offset": 0, "_length": 224}"#,
        "\n",
        r#"{"domain": "g", "date": "20260610", "time": "0000", "step": "3", "levtype": "pl", "levelist": "500", "param": "gh", "_offset": 224, "_length": 1024}"#,
    );
    let entries = parse_index(text, None).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].offset, 0);
    assert_eq!(entries[0].length, Some(224));
    assert_eq!(entries[0].var.as_deref(), Some("2t"));
    assert_eq!(entries[0].forecast_time.as_deref(), Some("3"));
    assert_eq!(
        entries[0].reference_date,
        Some(Utc.with_ymd_and_hms(2026, 6, 10, 0, 0, 0).unwrap())
    );
    assert_eq!(entries[1].level.as_deref(), Some("500"));
    assert_eq!(entries[1].keys["levtype"], "pl");
}
