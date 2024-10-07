extern crate gribberish;

use std::str::FromStr;

use gribberish::templates::product::parameters::{
    meteorological::TemperatureProduct, oceanographic::WavesProduct,
};

#[test]
fn test_from_abbrev_str() {
    let t = TemperatureProduct::from_str("TMP").unwrap();
    assert_eq!(t, TemperatureProduct::Temperature);

    let w = WavesProduct::from_str("WVDIR").unwrap();
    assert_eq!(w, WavesProduct::WindWaveDirection);

    assert!(TemperatureProduct::from_str("BAD STRING!").is_err());
}
