use chrono::{Duration, TimeZone, Utc};
use gribberish::{grib1::product_definition::Grib1ProductDefinitionSection, message::Message};

fn pds(time_range: u8, time_unit: u8, p1: u8, p2: u8) -> Vec<u8> {
    let mut data = vec![0; 28];
    data[2] = 28;
    data[3] = 2; // WMO parameter table
    data[6] = 255; // No predefined grid
    data[8] = 11; // Temperature
    data[9] = 1; // Surface
    data[12..17].copy_from_slice(&[24, 1, 31, 12, 0]);
    data[17..21].copy_from_slice(&[time_unit, p1, p2, time_range]);
    data[24] = 21; // Century
    data
}

fn assert_forecast(data: &[u8], expected: Duration) {
    let section = Grib1ProductDefinitionSection::from_data(data).unwrap();
    let reference = Utc.with_ymd_and_hms(2024, 1, 31, 12, 0, 0).unwrap();
    assert_eq!(section.forecast_duration().unwrap(), expected);
    assert_eq!(section.forecast_datetime().unwrap(), reference + expected);

    // Synthetic message: indicator, PDS, constant-field BDS, end marker.
    let mut message = b"GRIB\0\0\0\x01".to_vec();
    message.extend_from_slice(data);
    let mut bds = [0; 12];
    bds[2] = 12;
    bds[3] = 8; // Eight unused bits in the final octet
    message.extend_from_slice(&bds);
    message.extend_from_slice(b"7777");
    let length = message.len() as u32;
    message[4..7].copy_from_slice(&length.to_be_bytes()[1..]);
    let parsed = Message::from_data(&message, 0).unwrap();
    assert_eq!(parsed.forecast_date().unwrap(), reference + expected);
}

#[test]
fn extended_forecast_period_uses_both_octets() {
    for hours in [0u16, 1, 252, 255, 256, 258, 264, 300, 360, 511, 512, 65535] {
        let [p1, p2] = hours.to_be_bytes();
        assert_forecast(&pds(10, 1, p1, p2), Duration::hours(i64::from(hours)));
    }
}

#[test]
fn ordinary_forecast_period_uses_only_p1() {
    for hours in [0, 1, 252, 255] {
        assert_forecast(&pds(0, 1, hours, 0), Duration::hours(i64::from(hours)));
    }
    assert_forecast(&pds(0, 1, 6, 12), Duration::hours(6));
    assert_forecast(&pds(1, 1, 0, 0), Duration::zero());
}

#[test]
fn forecast_period_respects_grib1_time_units() {
    // WMO GRIB1 Code Table 4; code 13 differs from GRIB2.
    for (unit, seconds) in [
        (0, 60),
        (1, 3600),
        (2, 86400),
        (10, 10800),
        (11, 21600),
        (12, 43200),
        (13, 900),
        (14, 1800),
        (254, 1),
    ] {
        assert_forecast(&pds(0, unit, 6, 0), Duration::seconds(6 * seconds));
        assert_forecast(&pds(10, unit, 1, 2), Duration::seconds(258 * seconds));
    }
}

#[test]
fn unsupported_time_unit_returns_error() {
    for time_range in [0, 10] {
        let data = pds(time_range, 255, 1, 2);
        let section = Grib1ProductDefinitionSection::from_data(&data).unwrap();
        assert_eq!(
            section.forecast_duration().unwrap_err(),
            "Unsupported time unit: 255"
        );
    }
}
