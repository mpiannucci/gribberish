#![cfg(feature = "zarrs")]

extern crate gribberish;

use std::sync::Arc;

use gribberish::message::Message;
use zarrs::array::Array;
use zarrs::storage::store::MemoryStore;
use zarrs::storage::{StoreKey, WritableStorageTraits};

// GEFS 0.5° — first message (HGT) is a 361×720 global grid, lon 0..359.5. The
// antimeridian (180°) sits on column 360, so the wrap rolls by exactly 360.
const GEAVG: &str = "../test-data/geavg.t12z.pgrb2a.0p50.f000";
const GEAVG_SHAPE: (usize, usize) = (361, 720);
const GEAVG_ROLL: usize = 360;

/// The raw bytes of the first message in the GEAVG file.
fn first_message() -> Vec<u8> {
    let data = std::fs::read(GEAVG).expect("failed to read test data");
    let len = Message::from_data(&data, 0)
        .expect("failed to read GRIB message")
        .len();
    data[..len].to_vec()
}

/// Write a single-chunk array using the gribberish codec, store `chunk` as its
/// only chunk, and read it back through zarrs.
fn decode<T: zarrs::array::ElementOwned>(
    configuration: serde_json::Value,
    shape: &[usize],
    data_type: &str,
    chunk: Vec<u8>,
) -> Result<Vec<T>, zarrs::array::ArrayError> {
    let store = Arc::new(MemoryStore::new());
    let fill_value = if data_type.starts_with("float") {
        serde_json::json!("NaN")
    } else {
        serde_json::json!(0)
    };
    let metadata = serde_json::json!({
        "zarr_format": 3,
        "node_type": "array",
        "shape": shape,
        "data_type": data_type,
        "chunk_grid": {"name": "regular", "configuration": {"chunk_shape": shape}},
        "chunk_key_encoding": {"name": "default", "configuration": {"separator": "/"}},
        "fill_value": fill_value,
        "codecs": [{"name": "gribberish", "configuration": configuration}],
    });
    store
        .set(
            &StoreKey::new("arr/zarr.json").unwrap(),
            serde_json::to_vec(&metadata).unwrap().into(),
        )
        .unwrap();
    let chunk_key = format!("arr/c/{}", vec!["0"; shape.len()].join("/"));
    store
        .set(&StoreKey::new(chunk_key).unwrap(), chunk.into())
        .unwrap();

    let array = Array::open(store, "/arr").expect("failed to open array");
    array.retrieve_chunk::<Vec<T>>(&vec![0; shape.len()])
}

#[test]
fn decodes_data_like_message_data() {
    let raw = first_message();
    let expected = Message::from_data(&raw, 0).unwrap().data().unwrap();

    let decoded: Vec<f64> = decode(
        serde_json::json!({"var": "HGT"}),
        &[GEAVG_SHAPE.0, GEAVG_SHAPE.1],
        "float64",
        raw,
    )
    .unwrap();

    assert_eq!(decoded.len(), expected.len());
    for (d, e) in decoded.iter().zip(&expected) {
        assert!(d == e || (d.is_nan() && e.is_nan()));
    }
}

#[test]
fn decodes_to_float32() {
    let raw = first_message();
    let expected = Message::from_data(&raw, 0).unwrap().data().unwrap();

    let decoded: Vec<f32> = decode(
        serde_json::json!({}),
        &[GEAVG_SHAPE.0, GEAVG_SHAPE.1],
        "float32",
        raw,
    )
    .unwrap();

    assert_eq!(decoded[0], expected[0] as f32);
}

#[test]
fn adjust_longitude_range_rolls_data() {
    let raw = first_message();
    let shape = [GEAVG_SHAPE.0, GEAVG_SHAPE.1];
    let native: Vec<f64> = decode(
        serde_json::json!({"var": "HGT"}),
        &shape,
        "float64",
        raw.clone(),
    )
    .unwrap();
    let adjusted: Vec<f64> = decode(
        serde_json::json!({"var": "HGT", "adjust_longitude_range": true}),
        &shape,
        "float64",
        raw,
    )
    .unwrap();

    let (ny, nx) = GEAVG_SHAPE;
    for row in 0..ny {
        for col in 0..nx {
            let src = (col + GEAVG_ROLL) % nx;
            assert_eq!(adjusted[row * nx + col], native[row * nx + src]);
        }
    }
    assert_ne!(adjusted, native);
}

#[test]
fn adjust_longitude_range_wraps_longitude_coordinate() {
    let raw = first_message();
    let shape = [GEAVG_SHAPE.1];
    let native: Vec<f64> = decode(
        serde_json::json!({"var": "longitude"}),
        &shape,
        "float64",
        raw.clone(),
    )
    .unwrap();
    let wrapped: Vec<f64> = decode(
        serde_json::json!({"var": "longitude", "adjust_longitude_range": true}),
        &shape,
        "float64",
        raw,
    )
    .unwrap();

    assert_eq!((native[0], native[native.len() - 1]), (0.0, 359.5));
    assert_eq!((wrapped[0], wrapped[wrapped.len() - 1]), (-180.0, 179.5));
    assert!(wrapped.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn north_up_puts_northern_most_latitude_first() {
    let raw = first_message();
    let latitude: Vec<f64> = decode(
        serde_json::json!({"var": "latitude", "north_up": true}),
        &[GEAVG_SHAPE.0],
        "float64",
        raw,
    )
    .unwrap();

    assert_eq!(latitude[0], 90.0);
    assert!(latitude.windows(2).all(|w| w[0] > w[1]));
}

#[test]
fn mismatched_chunk_shape_is_an_error() {
    let result: Result<Vec<f64>, _> =
        decode(serde_json::json!({}), &[10, 10], "float64", first_message());
    assert!(result.is_err());
}

#[test]
fn unsupported_data_type_is_an_error() {
    let result: Result<Vec<i32>, _> = decode(
        serde_json::json!({}),
        &[GEAVG_SHAPE.0, GEAVG_SHAPE.1],
        "int32",
        first_message(),
    );
    assert!(result.is_err());
}
