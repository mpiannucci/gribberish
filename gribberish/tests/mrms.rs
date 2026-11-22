use gribberish::message::read_messages;

// 40 × 50 sub-area cut with wgrib2 from a real MRMS SeamlessHSR message (2026-11-16 07Z),
// repacked to PNG (DRT 5.41) with eccodes. Discipline 209 is the NSSL MRMS local table.
// Expected values are eccodes' decode of the cut file.
const MRMS_SEAMLESS_HSR: &[u8] = include_bytes!("../../test-data/mrms_seamlesshsr_subarea.grib2");

#[test]
fn read_mrms_seamless_hsr_message() {
    let message = read_messages(MRMS_SEAMLESS_HSR).next().unwrap();

    assert_eq!(message.discipline_value().unwrap(), 209);
    assert_eq!(message.category_value().unwrap(), 8);
    assert_eq!(message.parameter_value().unwrap(), 8);
    assert_eq!(
        message.parameter_index().unwrap(),
        "(multiradarmultisensor, 8, 8)"
    );
    assert_eq!(message.category().unwrap(), "quality");
    let parameter = message.parameter().unwrap();
    assert_eq!(parameter.abbrev, "SeamlessHSR");
    assert_eq!(parameter.unit, "dBZ");
    assert_eq!(parameter.name, "seamlesshsr");
    assert_eq!(message.variable_abbrev().unwrap(), "SeamlessHSR");
    assert_eq!(message.unit().unwrap(), "dBZ");

    assert_eq!(message.grid_dimensions().unwrap(), (50, 40));
    let values = message.data().unwrap();
    assert_eq!(values.len(), 40 * 50);
    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    for (actual, expected) in [
        (values[0], -99.0),
        (values[160], 24.4),
        (values[1021], 51.3),
        (min, -99.0),
        (max, 51.3),
        (mean, -56.9827),
    ] {
        assert!((actual - expected).abs() < 1e-6, "{actual} != {expected}");
    }
}
