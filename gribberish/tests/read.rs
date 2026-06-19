extern crate gribberish;

use gribberish::message::{read_messages, Message};
use gribberish::templates::product::tables::{DerivedForecastType, TypeOfStatisticalProcessing};
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
    let grib_data = read_grib_messages("../test-data/multi_1.at_10m.t12z.f147.grib2");
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
    let read_data = read_grib_messages("../test-data/hrrr.t06z.wrfsfcf01-UGRD.grib2");
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
    let read_data = read_grib_messages("../test-data/hrrr.t06z.wrfsfcf01-TMP.grib2");
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
fn read_spatial_differenced_complex_with_missing() {
    // GFS temperature on the potential-vorticity surface: complex packing with
    // 2nd-order spatial differencing AND primary missing values (large regions
    // of the grid are missing). Validated against cfgrib/eccodes. Regression
    // test for the missing-value handling -- previously the all-ones group
    // references were decoded as real data and the second-order reconstruction
    // diverged into ~2^31 garbage across 99% of the grid.
    let read_data =
        read_grib_messages("../test-data/gfs.t12z.pgrb2.0p25.f023-PV-TMP-missing.grib2");
    let message = read_messages(read_data.as_slice())
        .next()
        .expect("a message");
    let data = message.data().expect("decode");

    assert_eq!(data.len(), 1038240);
    let defined: Vec<f64> = data.iter().copied().filter(|v| v.is_finite()).collect();
    let missing = data.len() - defined.len();
    assert_eq!(defined.len(), 629160, "defined value count");
    assert_eq!(missing, 409080, "missing (NaN) value count");

    let min = defined.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = defined.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!((min - 184.98).abs() < 0.01, "min was {min}");
    assert!((max - 289.58).abs() < 0.01, "max was {max}");
    // first row is fully defined and constant in this message
    assert!(
        (data[0] - 248.6795898).abs() < 1e-4,
        "data[0] was {}",
        data[0]
    );
}

#[test]
fn read_simple_zerod() {
    let read_data = read_grib_messages("../test-data/hrrr.t06z.wrfsfcf01-CFRZR.grib2");
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
    let read_data = read_grib_messages("../test-data/gfs.t18z.pgrb2.0p25.f186-RH.grib2");
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
    let read_data = read_grib_messages("../test-data/meteofrance.mfwam.arome-SWELL.grib2");
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
    let read_data = read_grib_messages("../test-data/meteofrance.mfwam.arome-SWELL.grib2");
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
    let grib_data = read_grib_messages("../test-data/geavg.t12z.pgrb2a.0p50.f000");
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
    let grib_data = read_grib_messages("../test-data/era5-levels-members.grib");
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

    // Validate key format (message 0 is ensemble member 0)
    let key = msg_0.key().unwrap();
    assert_eq!(
        key, "z:201701010000:500 in mb:ens0:forecast",
        "Message 0 key"
    );

    // Validate data at multiple points
    let data_0 = msg_0.data().unwrap();
    assert_eq!(data_0.len(), 7320, "Message 0 data length");
    assert!((data_0[0] - 51169.703125).abs() < 0.01, "Message 0 data[0]");
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
fn read_grib1_ecmwf_table_128_soil_tiny_fixture() {
    let grib_data = read_grib_messages("../test-data/ecmwf_soil_8vars_tiny.grib1");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    assert_eq!(
        messages.len(),
        8,
        "Expected 8 messages in tiny ECMWF soil fixture"
    );

    let mut vars = std::collections::HashSet::new();
    for message in &messages {
        vars.insert(message.variable_abbrev().unwrap());
        assert_eq!(
            message.grid_dimensions().unwrap(),
            (4, 4),
            "Expected 4x4 tiny grid"
        );
        assert_eq!(
            message.data().unwrap().len(),
            16,
            "Expected 16 data points per message"
        );
    }

    let expected: std::collections::HashSet<String> = [
        "swvl1", "swvl2", "swvl3", "swvl4", "stl1", "stl2", "stl3", "stl4",
    ]
    .iter()
    .map(|v| (*v).to_string())
    .collect();
    assert_eq!(vars, expected, "Unexpected GRIB1 soil variable set");
}

#[test]
fn test_iterator_scans_past_padding() {
    // This test verifies that the MessageIterator correctly scans past
    // non-GRIB data (padding bytes, headers, etc.) to find all GRIB messages
    let grib_data = read_grib_messages("../test-data/era5-levels-members.grib");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    // The ERA5 file has 160 GRIB1 messages with 8-byte padding between each
    // (2 variables x 4 times x 2 levels x 10 ensemble members)
    assert_eq!(
        messages.len(),
        160,
        "Expected 160 messages in ERA5 GRIB1 file (iterator should scan past padding)"
    );

    // Every message has a unique key: the ensemble member number is part of
    // the key, so the 10 members no longer collide.
    let mut unique_keys = std::collections::HashSet::new();
    for message in &messages {
        let key = message.key().unwrap();
        unique_keys.insert(key);
    }

    assert_eq!(
        unique_keys.len(),
        160,
        "Expected 160 unique keys (2 vars x 4 times x 2 levels x 10 members)"
    );
}

#[test]
fn test_longitude_normalization() {
    // Test that longitude values are correctly computed and normalized for global grids
    // This validates the fix for longitude wrapping in the projection code

    // Test with geavg (GRIB2) - 0.5° global grid starting at 0°
    let grib_data = read_grib_messages("../test-data/geavg.t12z.pgrb2a.0p50.f000");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();
    assert!(!messages.is_empty(), "Expected at least one message");

    let msg = &messages[0];
    let projector = msg.latlng_projector().expect("Failed to get projector");
    let (lats, lngs) = projector.lat_lng();

    // Verify grid dimensions
    assert_eq!(lats.len(), 361, "Expected 361 latitude points");
    assert_eq!(lngs.len(), 720, "Expected 720 longitude points");

    // Verify latitude range (90 to -90)
    assert!(
        (lats[0] - 90.0).abs() < 0.001,
        "First latitude should be 90°"
    );
    assert!(
        (lats[360] - (-90.0)).abs() < 0.001,
        "Last latitude should be -90°"
    );

    // Verify longitude range (0 to 359.5)
    assert!(
        (lngs[0] - 0.0).abs() < 0.001,
        "First longitude should be 0°"
    );
    assert!(
        (lngs[719] - 359.5).abs() < 0.001,
        "Last longitude should be 359.5°"
    );

    // All longitudes should be in valid range [0, 360)
    for (i, lng) in lngs.iter().enumerate() {
        assert!(
            *lng >= 0.0 && *lng < 360.0,
            "Longitude[{}] = {} is out of valid range [0, 360)",
            i,
            lng
        );
    }

    // Verify monotonic increase in longitude (no wrapping issues for this grid)
    for i in 1..lngs.len() {
        assert!(
            lngs[i] > lngs[i - 1],
            "Longitudes should be monotonically increasing: lng[{}]={} > lng[{}]={}",
            i,
            lngs[i],
            i - 1,
            lngs[i - 1]
        );
    }
}

#[test]
fn test_longitude_normalization_era5_grib1() {
    // Test longitude normalization for GRIB1 ERA5 file
    let grib_data = read_grib_messages("../test-data/era5-levels-members.grib");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();
    assert!(!messages.is_empty(), "Expected at least one message");

    let msg = &messages[0];
    let projector = msg.latlng_projector().expect("Failed to get projector");
    let (lats, lngs) = projector.lat_lng();

    // Verify grid dimensions (61 lat x 120 lon at 3° resolution)
    assert_eq!(lats.len(), 61, "Expected 61 latitude points");
    assert_eq!(lngs.len(), 120, "Expected 120 longitude points");

    // Verify latitude range (90 to -90)
    assert!(
        (lats[0] - 90.0).abs() < 0.001,
        "First latitude should be 90°"
    );
    assert!(
        (lats[60] - (-90.0)).abs() < 0.001,
        "Last latitude should be -90°"
    );

    // Verify longitude range (0 to 357 at 3° steps)
    assert!(
        (lngs[0] - 0.0).abs() < 0.001,
        "First longitude should be 0°"
    );
    assert!(
        (lngs[119] - 357.0).abs() < 0.001,
        "Last longitude should be 357°"
    );

    // All longitudes should be in valid range [0, 360)
    for (i, lng) in lngs.iter().enumerate() {
        assert!(
            *lng >= 0.0 && *lng < 360.0,
            "Longitude[{}] = {} is out of valid range [0, 360)",
            i,
            lng
        );
    }
}

#[test]
fn read_nbm_lambert_specified_radius_projection() {
    // Regression test: NBM uses a Lambert Conformal grid whose earth shape is
    // "spherical with a producer-specified radius" (shape code 1). That radius
    // lives in the "radius of spherical earth" fields, not the major/minor axis
    // fields (which are set to missing). Reading the missing axis fields gave a
    // radius of ~2.5e-253 m, producing garbage lat/lon (and an inverse-project
    // panic for some grids). The first CONUS grid point must land near
    // 19.229°N, 233.72°E (-126.28°).
    let grib_data = read_grib_messages("../test-data/nbm-pwat-prob-above.grib2");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();
    assert!(!messages.is_empty(), "Expected at least one message");

    let msg = &messages[0];

    // The producer-specified spherical radius must be read from the radius
    // fields (6,371,200 m), not the missing major/minor axis fields.
    let params = msg.grid_template().unwrap().proj_params();
    assert!(
        (params["a"] - 6_371_200.0).abs() < 1.0,
        "a = {}",
        params["a"]
    );
    assert!(
        (params["b"] - 6_371_200.0).abs() < 1.0,
        "b = {}",
        params["b"]
    );

    let projector = msg.latlng_projector().expect("Failed to get projector");
    let (lats, lngs) = projector.lat_lng();
    assert_eq!(lats.len(), 1597 * 2345);
    assert_eq!(lngs.len(), 1597 * 2345);

    // First grid point: ~19.229°N, ~-126.28°.
    assert!((lats[0] - 19.229).abs() < 0.01, "first lat = {}", lats[0]);
    assert!(
        (lngs[0] - (-126.28)).abs() < 0.01,
        "first lng = {}",
        lngs[0]
    );

    // Every point must fall inside the physical NBM CONUS footprint — not the
    // degenerate values the bad radius produced (e.g. 90°N / -307°).
    for lat in &lats {
        assert!(
            (15.0..=60.0).contains(lat),
            "latitude {} outside CONUS range",
            lat
        );
    }
    for lng in &lngs {
        assert!(
            (-140.0..=-50.0).contains(lng),
            "longitude {} outside CONUS range",
            lng
        );
    }
}

#[test]
fn test_longitude_adjustment_geavg() {
    // GEFS 0.5° global grid (0..359.5): opt-in wrap to -180..180.
    let grib_data = read_grib_messages("../test-data/geavg.t12z.pgrb2a.0p50.f000");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();
    let msg = &messages[0];
    let projector = msg.latlng_projector().expect("Failed to get projector");

    let (lats, lngs) = projector.lat_lng_adjusted(true);

    // Latitudes untouched; longitudes wrapped, monotonic, in [-180, 180).
    assert_eq!(lats, projector.lat_lng().0);
    assert_eq!(lngs.len(), 720);
    assert!(
        (lngs[0] - (-180.0)).abs() < 0.001,
        "first lng should be -180"
    );
    assert!(
        (lngs[360] - 0.0).abs() < 0.001,
        "lng at index 360 should be 0"
    );
    assert!(
        (lngs[719] - 179.5).abs() < 0.001,
        "last lng should be 179.5"
    );
    for i in 1..lngs.len() {
        assert!(lngs[i] > lngs[i - 1], "longitudes must be monotonic");
        assert!((-180.0..180.0).contains(&lngs[i]), "lng out of range");
    }

    // Data is rolled by the same amount (360) so it stays aligned with coords.
    let native = msg.data().expect("Failed to decode data");
    let adjusted = projector.adjust_data_longitude(native.clone(), true);
    assert_eq!(adjusted.len(), native.len());
    let nx = 720usize;
    for (i, value) in adjusted.iter().enumerate() {
        let (row, col) = (i / nx, i % nx);
        let expected = native[row * nx + (col + 360) % nx];
        assert_eq!(*value, expected, "data not rolled correctly at {i}");
    }

    // Disabled / no-op path returns the originals unchanged.
    assert_eq!(projector.lat_lng_adjusted(false), projector.lat_lng());
    assert_eq!(
        projector.adjust_data_longitude(native.clone(), false),
        native
    );
}

#[test]
fn test_longitude_adjustment_era5_grib1() {
    // ERA5 GRIB1 3° global grid (0..357): wrap to -180..180, roll 60.
    let grib_data = read_grib_messages("../test-data/era5-levels-members.grib");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();
    let msg = &messages[0];
    let projector = msg.latlng_projector().expect("Failed to get projector");

    let (_, lngs) = projector.lat_lng_adjusted(true);
    assert_eq!(lngs.len(), 120);
    assert!(
        (lngs[0] - (-180.0)).abs() < 0.001,
        "first lng should be -180"
    );
    assert!(
        (lngs[60] - 0.0).abs() < 0.001,
        "lng at index 60 should be 0"
    );
    assert!((lngs[119] - 177.0).abs() < 0.001, "last lng should be 177");
    for i in 1..lngs.len() {
        assert!(lngs[i] > lngs[i - 1], "longitudes must be monotonic");
    }

    let native = msg.data().expect("Failed to decode data");
    let adjusted = projector.adjust_data_longitude(native.clone(), true);
    let nx = 120usize;
    for (i, value) in adjusted.iter().enumerate() {
        let (row, col) = (i / nx, i % nx);
        assert_eq!(
            *value,
            native[row * nx + (col + 60) % nx],
            "roll mismatch at {i}"
        );
    }
}

#[test]
fn read_hrrr_hpbl_parameter() {
    // Test the new PlanetaryBoundaryLayerHeight (HPBL) meteorological parameter
    // disc=0 (meteorological), cat=3 (mass), num=18
    let read_data = read_grib_messages("../test-data/hrrr.t00z.wrfsfcf00-HPBL.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    // Validate the new parameter metadata
    assert_eq!(message.variable_abbrev().unwrap(), "HPBL");
    assert_eq!(
        message.variable_name().unwrap(),
        "planetaryboundarylayerheight"
    );
    assert_eq!(message.unit().unwrap(), "m");

    // Validate grid dimensions (HRRR 3km CONUS grid)
    let dims = message.grid_dimensions().unwrap();
    assert_eq!(dims, (1059, 1799));

    // Validate data can be unpacked
    let data = message.data().unwrap();
    assert_eq!(data.len(), 1905141);

    // Validate specific data values (planetary boundary layer height in meters)
    assert!((data[0] - 741.0781190395355).abs() < 0.001, "data[0]");
    assert!((data[1000] - 727.7656190395355).abs() < 0.001, "data[1000]");
    assert!(
        (data[100000] - 678.6406190395355).abs() < 0.001,
        "data[100000]"
    );
}

#[test]
fn read_hrrr_cfrzr_metadata() {
    // Validate that the existing CFRZR test file resolves correct metadata
    // This file uses categorical freezing rain (disc=0, cat=1, num=34)
    let read_data = read_grib_messages("../test-data/hrrr.t06z.wrfsfcf01-CFRZR.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "CFRZR");
    assert_eq!(message.variable_name().unwrap(), "categoricalfreezingrain");
    assert_eq!(message.unit().unwrap(), "BOOL");

    let dims = message.grid_dimensions().unwrap();
    assert_eq!(dims, (1059, 1799));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1905141);
    assert!((data[1000] - 0.0).abs() < 0.0000001);
}

#[test]
fn read_hrrr_atmospheric_chemistry() {
    // GRIB2 discipline 0 (meteorological), category 20 (atmospheric chemical constituents),
    // parameter 0 (mass density). Reference: GRIB2 Table 4.2-0-20
    // Source: HRRR wrfprsf00 smoke/dust fields
    let read_data = read_grib_messages("../test-data/hrrr.t00z.wrfprsf00-atmo-chem.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "MASSDEN");
    assert_eq!(message.variable_name().unwrap(), "massdensity");
    assert_eq!(message.unit().unwrap(), "kg m-3");
    assert_eq!(message.grid_dimensions().unwrap(), (1059, 1799));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1905141);
}

#[test]
fn read_hrrr_hydrology() {
    // GRIB2 discipline 1 (hydrology), category 0 (basic), parameter 6 (storm surface runoff).
    // Reference: GRIB2 Table 4.2-1-0
    // Source: HRRR wrfprsf00 surface runoff fields
    let read_data = read_grib_messages("../test-data/hrrr.t00z.wrfprsf00-hydrology.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "SSRUN");
    assert_eq!(message.variable_name().unwrap(), "stormsurfacerunoff");
    assert_eq!(message.unit().unwrap(), "kg m-2");
    assert_eq!(message.grid_dimensions().unwrap(), (1059, 1799));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1905141);
    // Constant field — all values should be 0.0
    assert!((data[0] - 0.0).abs() < 0.0000001, "data[0]");
}

#[test]
fn read_hrrr_space_products() {
    // GRIB2 discipline 3 (space products), category 192 (NCEP local),
    // parameter 1 (simulated brightness temp GOES 12 ch 3).
    // Reference: GRIB2 Table 4.2-3-192 (NCEP local use)
    // Source: HRRR wrfprsf00 simulated satellite fields
    let read_data = read_grib_messages("../test-data/hrrr.t00z.wrfprsf00-space.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "SBT123");
    assert_eq!(
        message.variable_name().unwrap(),
        "simulatedbrightnesstempgoes12ch3"
    );
    assert_eq!(message.unit().unwrap(), "K");
    assert_eq!(message.grid_dimensions().unwrap(), (1059, 1799));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1905141);
    assert!((data[0] - 246.05).abs() < 0.01, "data[0]");
    assert!((data[1000] - 250.81).abs() < 0.01, "data[1000]");
}

#[test]
fn read_hrrr_template_8_time_interval() {
    // GRIB2 Product Definition Template 4.8: average/accumulation in statistical time interval.
    // Reference: GRIB2 Table 4.0, Template 4.8
    // disc=0/cat=2/param=220 is NCEP local (not in standard tables), so var_name is "missing".
    // Source: HRRR wrfprsf00 time-interval accumulated fields
    let read_data = read_grib_messages("../test-data/hrrr.t00z.wrfprsf00-template8.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    // Template 8 should parse successfully even for unknown parameters
    assert_eq!(message.product_template_id().unwrap(), 8);
    assert_eq!(message.grid_dimensions().unwrap(), (1059, 1799));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1905141);
}

#[test]
fn read_grib1_ecmwf_visibility() {
    // GRIB1 WMO standard table 2, parameter 20 (visibility).
    // Reference: WMO GRIB1 Code Table 2 (FM 92-XII Ext. GRIB, Table 2)
    // Source: IFS/ECMWF global forecast, 0.125° grid
    let read_data = read_grib_messages("../test-data/ecmwf.t00z.pgrb.0p125.f000-vis.grib1");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "vis");
    assert_eq!(message.variable_name().unwrap(), "Visibility");
    assert_eq!(message.unit().unwrap(), "m");
    assert_eq!(message.grid_dimensions().unwrap(), (1441, 2880));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 4150080);
    assert!((data[0] - 80459.04).abs() < 0.01, "data[0]");
}

#[test]
fn read_grib1_ecmwf_table_140_wave() {
    // GRIB1 ECMWF local table 140, parameter 229 (significant height of combined wind waves
    // and swell). Reference: ECMWF Parameter Database, Table 140
    // Source: IFS/ECMWF global forecast wave model fields, 0.125° grid
    let read_data = read_grib_messages("../test-data/ecmwf.t00z.pgrb.0p125.f000-wave.grib1");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "swh");
    assert_eq!(
        message.variable_name().unwrap(),
        "Significant height of combined wind waves and swell"
    );
    assert_eq!(message.unit().unwrap(), "m");
    assert_eq!(message.grid_dimensions().unwrap(), (1441, 2880));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 4150080);
    // First data point is over land (NaN/missing)
    assert!(data[0].is_nan(), "data[0] should be NaN (over land)");
}

#[test]
fn read_aifs_single_temperature() {
    // ECMWF AIFS single deterministic forecast, 0.25° global grid.
    // GRIB2 Product Definition Template 4.0 (analysis/forecast at horizontal level).
    // Discipline 0 (meteorological), category 0 (temperature), parameter 0 (temperature).
    // Reference: GRIB2 Table 4.2-0-0
    // Source: public ECMWF AIFS data (s3://ecmwf-forecasts/)
    let read_data = read_grib_messages("../test-data/aifs-single-t500.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "TMP");
    assert_eq!(message.variable_name().unwrap(), "temperature");
    assert_eq!(message.unit().unwrap(), "K");
    assert_eq!(message.grid_dimensions().unwrap(), (721, 1440));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1038240);

    // Values cross-verified against eccodes (max_diff = 0.0)
    assert!((data[0] - 236.256073).abs() < 0.001, "data[0]");
    assert!((data[1000] - 236.256073).abs() < 0.001, "data[1000]");
    assert!((data[100000] - 236.677948).abs() < 0.001, "data[100000]");
}

#[test]
fn read_aifs_ensemble_temperature() {
    // ECMWF AIFS ensemble control forecast, 0.25° global grid.
    // GRIB2 Product Definition Template 4.1 (individual ensemble forecast at horizontal level).
    // Discipline 0 (meteorological), category 0 (temperature), parameter 0 (temperature).
    // Reference: GRIB2 Table 4.2-0-0, Template 4.1
    // Source: public ECMWF AIFS ensemble data (s3://ecmwf-forecasts/)
    let read_data = read_grib_messages("../test-data/aifs-ens-cf-t500.grib2");
    let mut messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 1);

    let message = messages.pop().unwrap();

    assert_eq!(message.variable_abbrev().unwrap(), "TMP");
    assert_eq!(message.variable_name().unwrap(), "temperature");
    assert_eq!(message.unit().unwrap(), "K");
    assert_eq!(message.product_template_id().unwrap(), 1);
    assert_eq!(message.grid_dimensions().unwrap(), (721, 1440));

    let data = message.data().unwrap();
    assert_eq!(data.len(), 1038240);

    // Values cross-verified against eccodes (max_diff = 0.0)
    assert!((data[0] - 236.822784).abs() < 0.001, "data[0]");
    assert!((data[1000] - 236.822784).abs() < 0.001, "data[1000]");
    assert!((data[100000] - 233.666534).abs() < 0.001, "data[100000]");
}

#[test]
fn read_ecmwf_ifs_oper_surface() {
    // ECMWF IFS operational 0.25° global analysis, surface fields.
    // Tests ECMWF-specific parameters within standard WMO disciplines:
    //   - Snow albedo (d=0/c=19/p=192, ECMWF local-use)
    //   - Precipitation type (d=0/c=1/p=19, WMO table 4.2-0-1)
    //   - Snow density (d=0/c=1/p=61, WMO table 4.2-0-1)
    //   - Snow depth water equivalent (d=0/c=1/p=254, ECMWF local-use)
    //   - Total cloud cover (d=0/c=6/p=192, ECMWF local-use)
    // Data from public ECMWF forecasts: s3://ecmwf-forecasts/
    let read_data = read_grib_messages("../test-data/ecmwf-ifs-oper-surface.grib2");
    let messages = read_messages(read_data.as_slice()).collect::<Vec<Message>>();
    assert_eq!(messages.len(), 5);

    // Snow albedo
    let asn = &messages[0];
    assert_eq!(asn.variable_abbrev().unwrap(), "ASN");
    assert_eq!(asn.variable_name().unwrap(), "snowalbedo");
    assert_eq!(asn.grid_dimensions().unwrap(), (721, 1440));
    let data = asn.data().unwrap();
    assert!((data[0] - 0.849609).abs() < 0.001, "asn data[0]");

    // Precipitation type
    let ptype = &messages[1];
    assert_eq!(ptype.variable_abbrev().unwrap(), "PTYPE");
    assert_eq!(ptype.variable_name().unwrap(), "precipitationtype");
    let data = ptype.data().unwrap();
    assert!((data[0] - 5.0).abs() < 0.001, "ptype data[0]");

    // Snow density
    let rsn = &messages[2];
    assert_eq!(rsn.variable_abbrev().unwrap(), "RSN");
    assert_eq!(rsn.variable_name().unwrap(), "snowdensity");
    let data = rsn.data().unwrap();
    assert!((data[0] - 100.005066).abs() < 0.001, "rsn data[0]");

    // Snow depth water equivalent
    let sd = &messages[3];
    assert_eq!(sd.variable_abbrev().unwrap(), "SD");
    assert_eq!(sd.variable_name().unwrap(), "snowdepthwaterequivalent");

    // Total cloud cover (ECMWF local-use d=0/c=6/p=192)
    let tcc = &messages[4];
    assert_eq!(tcc.variable_abbrev().unwrap(), "TCC");
    assert_eq!(tcc.variable_name().unwrap(), "totalcloudcoverecmwf");
    let data = tcc.data().unwrap();
    assert!((data[0] - 0.109375).abs() < 0.001, "tcc data[0]");
}

#[test]
fn read_percentile_and_probability_templates() {
    // Subset with PDTs 9 (probability), 10 (percentile), and 12 (derived ensemble).
    // Tests that percentile and probability messages are parsed with unique keys
    // and correct metadata.
    // Fixture: 10 messages with constant data, 0.5° global grid.
    let grib_data = read_grib_messages("../test-data/s2s-pdt9-pdt10-pdt12.grib2");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    assert_eq!(messages.len(), 10, "Expected 10 messages in subset");

    // --- PDT 12: Derived ensemble (messages 0, 1) ---
    let msg_0 = &messages[0];
    assert_eq!(msg_0.variable_abbrev().unwrap(), "TMP");
    assert_eq!(msg_0.product_template_id().unwrap(), 12);
    assert_eq!(msg_0.grid_dimensions().unwrap(), (361, 720));
    assert_eq!(msg_0.percentile_value().unwrap(), None);
    assert_eq!(msg_0.probability_type().unwrap(), None);
    let data = msg_0.data().unwrap();
    assert_eq!(data.len(), 259920);
    // Constant-value packing: all data points are identical
    assert!(
        data.iter().all(|&v| (v - data[0]).abs() < 0.001),
        "All data should be constant"
    );

    // --- PDT 10: Percentile (messages 2, 3, 4) ---
    let pctl_1 = &messages[2];
    assert_eq!(pctl_1.product_template_id().unwrap(), 10);
    assert_eq!(pctl_1.percentile_value().unwrap(), Some(1));

    let pctl_50 = &messages[3];
    assert_eq!(pctl_50.product_template_id().unwrap(), 10);
    assert_eq!(pctl_50.percentile_value().unwrap(), Some(50));

    let pctl_99 = &messages[4];
    assert_eq!(pctl_99.product_template_id().unwrap(), 10);
    assert_eq!(pctl_99.percentile_value().unwrap(), Some(99));

    // --- PDT 9: Probability (messages 5, 6, 7) ---
    let prob_above = &messages[5];
    assert_eq!(prob_above.product_template_id().unwrap(), 9);
    assert!(prob_above.probability_type().unwrap().is_some());
    assert!(prob_above.forecast_probability_number().unwrap().is_some());

    let prob_avg = &messages[6];
    assert_eq!(prob_avg.product_template_id().unwrap(), 9);

    let prob_limits = &messages[7];
    assert_eq!(prob_limits.product_template_id().unwrap(), 9);
    // This message has probability type 10 (between limits inclusive)
    // with lower=1, upper=10
    assert_eq!(prob_limits.probability_lower_limit().unwrap(), Some(1.0));
    assert_eq!(prob_limits.probability_upper_limit().unwrap(), Some(10.0));

    // --- PDT 9: Probability with varying thresholds (messages 8, 9) ---
    // Same variable and probability type as message 7, but different limits.
    // These exercise the threshold coordinate dimension.
    let prob_thresh_3 = &messages[8];
    assert_eq!(prob_thresh_3.product_template_id().unwrap(), 9);
    assert_eq!(prob_thresh_3.probability_lower_limit().unwrap(), Some(3.0));
    assert_eq!(prob_thresh_3.probability_upper_limit().unwrap(), Some(3.0));

    let prob_thresh_10 = &messages[9];
    assert_eq!(prob_thresh_10.product_template_id().unwrap(), 9);
    assert_eq!(
        prob_thresh_10.probability_lower_limit().unwrap(),
        Some(10.0)
    );
    assert_eq!(
        prob_thresh_10.probability_upper_limit().unwrap(),
        Some(10.0)
    );

    // --- All 10 messages should have unique keys ---
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
    assert_eq!(
        dups.len(),
        0,
        "Found {} duplicate keys: {:?}",
        dups.len(),
        dups
    );
}

#[test]
fn read_pdt107_anomaly_with_reference() {
    // Fixture: 4 messages with zeroed data, 0.5° global grid.
    // Messages 0-1: PDT 12 (derived ensemble time interval)
    // Messages 2-3: PDT 107 (same, with reference to normal — anomaly)
    // Messages 0 and 2 share the same variable/level/stat/derived type
    // and would collide without the :anom key suffix.
    let grib_data = read_grib_messages("../test-data/s2s-pdt12-pdt107-anomaly.grib2");
    let messages = read_messages(grib_data.as_slice()).collect::<Vec<Message>>();

    assert_eq!(messages.len(), 4, "Expected 4 messages in fixture");

    // --- PDT 12: TMP 2m Max UnweightedMean ---
    let pdt12_max = &messages[0];
    assert_eq!(pdt12_max.product_template_id().unwrap(), 12);
    assert_eq!(pdt12_max.variable_abbrev().unwrap(), "TMP");
    assert!(!pdt12_max.is_anomaly().unwrap());

    // --- PDT 12: TMP 2m Avg UnweightedMean ---
    let pdt12_avg = &messages[1];
    assert_eq!(pdt12_avg.product_template_id().unwrap(), 12);
    assert_eq!(pdt12_avg.variable_abbrev().unwrap(), "TMP");
    assert!(!pdt12_avg.is_anomaly().unwrap());

    // --- PDT 107: TMP 2m Max UnweightedMean (anomaly) ---
    let pdt107_max = &messages[2];
    assert_eq!(pdt107_max.product_template_id().unwrap(), 107);
    assert_eq!(pdt107_max.variable_abbrev().unwrap(), "TMP");
    assert!(pdt107_max.is_anomaly().unwrap());
    assert_eq!(
        pdt107_max.derived_forecast_type().unwrap(),
        Some(DerivedForecastType::UnweightedMean)
    );
    assert_eq!(
        pdt107_max.statistical_process_type().unwrap(),
        Some(TypeOfStatisticalProcessing::Maximum)
    );
    let data = pdt107_max.data().unwrap();
    assert_eq!(data.len(), 259920);

    // --- PDT 107: TMP 1000hPa Avg UnweightedMean (anomaly) ---
    let pdt107_avg = &messages[3];
    assert_eq!(pdt107_avg.product_template_id().unwrap(), 107);
    assert_eq!(pdt107_avg.variable_abbrev().unwrap(), "TMP");
    assert!(pdt107_avg.is_anomaly().unwrap());
    assert_eq!(
        pdt107_avg.derived_forecast_type().unwrap(),
        Some(DerivedForecastType::UnweightedMean)
    );
    assert_eq!(
        pdt107_avg.statistical_process_type().unwrap(),
        Some(TypeOfStatisticalProcessing::Average)
    );

    // --- All 4 messages should have unique keys ---
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
    assert_eq!(
        dups.len(),
        0,
        "Found {} duplicate keys: {:?}",
        dups.len(),
        dups
    );

    // Verify the anomaly key contains :anom and the non-anomaly does not
    let pdt12_key = pdt12_max.key().unwrap();
    let pdt107_key = pdt107_max.key().unwrap();
    assert!(
        !pdt12_key.contains(":anom"),
        "PDT 12 key should not contain :anom"
    );
    assert!(
        pdt107_key.contains(":anom"),
        "PDT 107 key should contain :anom"
    );
}
