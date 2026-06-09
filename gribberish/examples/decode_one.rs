extern crate gribberish;

use gribberish::message::MessageIterator;
use std::env;
use std::fs::File;
use std::io::Read;

fn main() {
    let path = env::args().nth(1).expect("pass a grib2 file");
    let mut raw = Vec::new();
    File::open(&path).unwrap().read_to_end(&mut raw).unwrap();

    let msg = MessageIterator::from_data(raw.as_slice(), 0)
        .next()
        .expect("no message");

    let drt = msg.data_template_number().unwrap();
    let n = msg.data_point_count().unwrap();
    let data = msg.data().expect("decode failed");

    let mut absurd = 0usize;
    let mut maxabs = 0.0f64;
    for &x in &data {
        if x.is_finite() {
            if x.abs() > maxabs {
                maxabs = x.abs();
            }
            if x.abs() > 1e6 {
                absurd += 1;
            }
        }
    }
    println!("rust crate: drt={drt} npoints={n} len={}", data.len());
    println!("|max|={maxabs:.4e}  absurd(>1e6)={absurd}/{}", data.len());
    println!("first5={:?}", &data[..5]);
}
