extern crate gribberish;

use gribberish::message::{read_messages, Message};
use std::time::Instant;
use std::vec::Vec;

use std::{fs::File, io::Read};

pub fn read_grib_messages(path: &str) -> Vec<u8> {
    let mut grib_file = File::open(path).expect("file not found");

    let mut raw_grib_data = Vec::new();
    grib_file
        .read_to_end(&mut raw_grib_data)
        .expect("failed to read raw grib2 data");

    raw_grib_data
}

#[test]
fn read_jpeg() {
    let grib_data = read_grib_messages("tests/data/multi_1.at_10m.t12z.f147.grib2");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    let message_count = messages.len();
    assert_ne!(message_count, 0);

    let mut keys = Vec::new();
    let mut dups = Vec::new();

    let mut durations = Vec::new();

    for message in messages {
        let key = message.key();
        assert!(key.is_ok());
        let key = key.unwrap();

        if keys.contains(&key) {
            dups.push(key);
        } else {
            keys.push(key);
        }

        let start = Instant::now();
        let data = message.data();
        let end = Instant::now();
        assert!(data.is_ok());
        durations.push(end.duration_since(start));
    }

    let duration_sum: u128 = durations.iter().map(|d| d.as_millis()).sum();
    let duration_mean = duration_sum / message_count as u128;
    println!("jpeg unpacking data() took an average of {duration_mean}ms per message");
}

#[test]
fn read_simple() {
    let read_data = read_grib_messages("tests/data/hrrr.t06z.wrfsfcf01-UGRD.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop();
    assert!(message.is_some());
    let message = message.unwrap();

    println!("{:?}", message.grid_dimensions().unwrap());

    let start = Instant::now();
    let data = message.data();
    let end = Instant::now();
    assert!(data.is_ok());
    let data = data.unwrap();
    println!(
        "simple unpacking data() took {:?} for {} data points",
        end.duration_since(start),
        data.len()
    );
    assert!((data[1000] - -4.46501350402832).abs() < 0.0000001);
}

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
    println!(
        "spatial complex unpacking data() took {:?} for {} data points",
        end.duration_since(start),
        data.len()
    );
    assert!((data[1000] - 303.7372741699219).abs() < 0.0000001);
}

#[test]
fn read_simple_zerod() {
    let read_data = read_grib_messages("tests/data/hrrr.t06z.wrfsfcf01-CFRZR.grib2");
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
    println!(
        "simple zero unpacking data() took {:?} for {} data points",
        end.duration_since(start),
        data.len()
    );
    assert!((data[1000] - 0.0).abs() < 0.0000001);
}

#[test]
fn read_complex_zerod() {
    let read_data = read_grib_messages("tests/data/gfs.t18z.pgrb2.0p25.f186-RH.grib2");
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
    println!(
        "spatial complex zero unpacking data() took {:?} for {} data points",
        end.duration_since(start),
        data.len()
    );
}

#[cfg(not(feature = "libaec"))]
#[test]
fn read_ccsds_without_libaec_should_work() {
    let read_data = read_grib_messages("tests/data/meteofrance.mfwam.arome-SWELL.grib2");
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
    println!(
        "pure rust ccsds unpacking data() took {:?} for {} data points",
        end.duration_since(start),
        data.len()
    );
}

#[cfg(feature = "libaec")]
#[test]
fn read_ccsds_with_libaec_should_work() {
    let read_data = read_grib_messages("tests/data/meteofrance.mfwam.arome-SWELL.grib2");
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
    println!(
        "ccsds unpacking data() took {:?} for {} data points",
        end.duration_since(start),
        data.len()
    );
}

#[test]
fn read_ensemble_average() {
    let grib_data = read_grib_messages("tests/data/geavg.t12z.pgrb2a.0p50.f000");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    // Verify we have the expected number of messages
    assert_eq!(
        messages.len(),
        71,
        "Expected 71 messages in ensemble average file"
    );

    // Test first message (HGT - Geopotential Height)
    let msg_0 = &messages[0];

    // Validate metadata
    assert_eq!(
        msg_0.variable_abbrev().unwrap(),
        "HGT",
        "Message 0 variable abbrev"
    );
    assert_eq!(
        msg_0.variable_name().unwrap(),
        "geopotentialheight",
        "Message 0 variable name"
    );
    assert_eq!(msg_0.unit().unwrap(), "gpm", "Message 0 unit");
    assert_eq!(
        msg_0.grid_dimensions().unwrap(),
        (361, 720),
        "Message 0 grid dimensions"
    );

    // Validate derived forecast type is UnweightedMean for ensemble average
    let derived_type = msg_0.derived_forecast_type().unwrap();
    assert!(
        derived_type.is_some(),
        "Message 0 should have derived forecast type"
    );
    assert_eq!(
        format!("{:?}", derived_type.unwrap()),
        "UnweightedMean",
        "Message 0 derived forecast type"
    );

    // Validate fixed surface
    let (surface_type, surface_value) = msg_0.first_fixed_surface().unwrap();
    assert_eq!(
        format!("{:?}", surface_type),
        "IsobaricSurface",
        "Message 0 first fixed surface type"
    );
    assert_eq!(
        surface_value,
        Some(1000.0),
        "Message 0 first fixed surface value"
    );

    // Validate data at multiple random points
    let data_0 = msg_0.data().unwrap();
    assert_eq!(data_0.len(), 259920, "Message 0 data length");
    assert!((data_0[0] - 29723.6525).abs() < 0.001, "Message 0 data[0]");
    assert!(
        (data_0[100] - 29723.6525).abs() < 0.001,
        "Message 0 data[100]"
    );
    assert!(
        (data_0[1000] - 29715.4525).abs() < 0.001,
        "Message 0 data[1000]"
    );

    // Test second message (TMP - Temperature)
    let msg_1 = &messages[1];

    // Validate metadata
    assert_eq!(
        msg_1.variable_abbrev().unwrap(),
        "TMP",
        "Message 1 variable abbrev"
    );
    assert_eq!(
        msg_1.variable_name().unwrap(),
        "temperature",
        "Message 1 variable name"
    );
    assert_eq!(msg_1.unit().unwrap(), "K", "Message 1 unit");
    assert_eq!(
        msg_1.grid_dimensions().unwrap(),
        (361, 720),
        "Message 1 grid dimensions"
    );
    assert_eq!(
        msg_1.product_template_id().unwrap(),
        2,
        "Message 1 product template ID"
    );

    // Validate derived forecast type
    let derived_type_1 = msg_1.derived_forecast_type().unwrap();
    assert!(
        derived_type_1.is_some(),
        "Message 1 should have derived forecast type"
    );
    assert_eq!(
        format!("{:?}", derived_type_1.unwrap()),
        "UnweightedMean",
        "Message 1 derived forecast type"
    );

    // Validate data at multiple random points
    let data_1 = msg_1.data().unwrap();
    assert_eq!(data_1.len(), 259920, "Message 1 data length");
    assert!(
        (data_1[0] - 205.70000000000002).abs() < 0.001,
        "Message 1 data[0]"
    );
    assert!(
        (data_1[100] - 205.70000000000002).abs() < 0.001,
        "Message 1 data[100]"
    );
    assert!(
        (data_1[1000] - 205.60000000000002).abs() < 0.001,
        "Message 1 data[1000]"
    );

    // Test third message (RH - Relative Humidity)
    let msg_2 = &messages[2];

    // Validate metadata
    assert_eq!(
        msg_2.variable_abbrev().unwrap(),
        "RH",
        "Message 2 variable abbrev"
    );
    assert_eq!(
        msg_2.variable_name().unwrap(),
        "relativehumidity",
        "Message 2 variable name"
    );
    assert_eq!(msg_2.unit().unwrap(), "%", "Message 2 unit");

    // Validate data at multiple random points
    let data_2 = msg_2.data().unwrap();
    assert_eq!(data_2.len(), 259920, "Message 2 data length");
    assert!((data_2[0] - 1.26).abs() < 0.001, "Message 2 data[0]");
    assert!((data_2[100] - 1.26).abs() < 0.001, "Message 2 data[100]");
    assert!((data_2[1000] - 1.26).abs() < 0.001, "Message 2 data[1000]");
    assert!((data_2[5000] - 1.5).abs() < 0.001, "Message 2 data[5000]");
    assert!(
        (data_2[10000] - 1.34).abs() < 0.001,
        "Message 2 data[10000]"
    );
    assert!(
        (data_2[data_2.len() - 1] - 0.01).abs() < 0.001,
        "Message 2 data[last]"
    );

    // Verify all messages have unique keys (no duplicates)
    let mut keys = Vec::new();
    let mut dups = Vec::new();

    for message in &messages {
        let key = message.key().unwrap();
        if keys.contains(&key) {
            dups.push(key);
        } else {
            keys.push(key);
        }
    }

    assert_eq!(dups.len(), 0, "Found {} duplicate keys", dups.len());
}

#[test]
fn read_grib1_era5_levels_members() {
    let grib_data = read_grib_messages("tests/data/era5-levels-members.grib");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    // Verify we have the expected number of messages
    // Note: The iterator now scans for GRIB magic, finding all 160 messages in the file
    // (2 variables x 4 times x 2 levels x 10 ensemble members), but this test validates
    // just the first message's data integrity
    assert!(
        messages.len() >= 1,
        "Expected at least 1 message in ERA5 GRIB1 file"
    );

    // Test first message (Geopotential)
    let msg_0 = &messages[0];

    // Validate metadata
    assert_eq!(
        msg_0.variable_abbrev().unwrap(),
        "z",
        "Message 0 variable abbrev"
    );
    assert_eq!(
        msg_0.variable_name().unwrap(),
        "Geopotential",
        "Message 0 variable name"
    );
    assert_eq!(msg_0.unit().unwrap(), "m2 s-2", "Message 0 unit");
    assert_eq!(
        msg_0.grid_dimensions().unwrap(),
        (61, 120),
        "Message 0 grid dimensions"
    );

    // Validate key format
    let key = msg_0.key().unwrap();
    assert_eq!(
        key,
        "z:201701010000:500 in mb:forecast",
        "Message 0 key"
    );

    // Validate data at multiple points
    let data_0 = msg_0.data().unwrap();
    assert_eq!(data_0.len(), 7320, "Message 0 data length");
    assert!(
        (data_0[0] - 51169.703125).abs() < 0.01,
        "Message 0 data[0]"
    );
    assert!(
        (data_0[100] - 51169.703125).abs() < 0.01,
        "Message 0 data[100]"
    );
    assert!(
        (data_0[1000] - 49097.703125).abs() < 0.01,
        "Message 0 data[1000]"
    );

}

#[test]
fn test_iterator_scans_past_padding() {
    // This test verifies that the MessageIterator correctly scans past
    // non-GRIB data (padding bytes, headers, etc.) to find all GRIB messages
    let grib_data = read_grib_messages("tests/data/era5-levels-members.grib");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    // The ERA5 file has 160 GRIB1 messages with 8-byte padding between each
    // (2 variables x 4 times x 2 levels x 10 ensemble members)
    assert_eq!(
        messages.len(),
        160,
        "Expected 160 messages in ERA5 GRIB1 file (iterator should scan past padding)"
    );

    // Verify we have 16 unique variable/time/level combinations
    let mut unique_keys = std::collections::HashSet::new();
    for message in &messages {
        let key = message.key().unwrap();
        unique_keys.insert(key);
    }

    assert_eq!(
        unique_keys.len(),
        16,
        "Expected 16 unique keys (2 vars x 4 times x 2 levels)"
    );
}
