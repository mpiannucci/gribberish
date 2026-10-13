use gribberish::message::read_messages;
use gribberish::message_metadata::{scan_message_metadata, MessageMetadata};
use gribberish::templates::product::tables::FixedSurfaceType;

// Two ECMWF-style GRIB2 messages on hybrid model levels, built with eccodes from its
// `regular_ll_sfc_grib2` sample (16 × 31, centre ecmf, PDT 4.0, simple packing) and the
// real IFS L91 `pv` array from its `regular_gg_ml_grib2` sample: `cat` (paramId 260290,
// 0/19/29) on level 100 and `lnsp` (paramId 152, 0/3/25) on level 1. Expected numbers are
// eccodes' own decode of the written file.
const ECMWF_ML: &[u8] = include_bytes!("../../test-data/ecmwf_ml_cat_lnsp_tiny.grib2");

const PV_LEN: usize = 184;
const PV_HEAD: [f64; 3] = [0.0, 2.000040054321289, 3.980832099914551];

fn metadata() -> Vec<MessageMetadata> {
    read_messages(ECMWF_ML)
        .map(|m| MessageMetadata::try_from(&m).unwrap())
        .collect()
}

fn assert_close(actual: f64, expected: f64, tol: f64, what: &str) {
    assert!(
        (actual - expected).abs() < tol,
        "{what}: {actual} != {expected}"
    );
}

#[test]
fn ecmwf_cat_and_lnsp_are_named_and_not_dropped() {
    let meta = metadata();
    assert_eq!(meta.len(), 2);

    let cat = &meta[0];
    assert_eq!(cat.var, "CAT");
    assert_eq!(
        (
            cat.discipline_value,
            cat.category_value,
            cat.parameter_value
        ),
        (0, 19, 29)
    );
    assert_eq!(cat.units, "m2/3s-1");
    assert_eq!(cat.first_fixed_surface_type, FixedSurfaceType::HybridLevel);
    assert_eq!(cat.first_fixed_surface_value, Some(100.0));

    let lnsp = &meta[1];
    assert_eq!(lnsp.var, "LNSP");
    assert_eq!(
        (
            lnsp.discipline_value,
            lnsp.category_value,
            lnsp.parameter_value
        ),
        (0, 3, 25)
    );
    assert_eq!(lnsp.first_fixed_surface_type, FixedSurfaceType::HybridLevel);
    assert_eq!(lnsp.first_fixed_surface_value, Some(1.0));

    // scan_message_metadata silently drops messages whose metadata fails to parse.
    assert_eq!(scan_message_metadata(ECMWF_ML).len(), 2);
}

#[test]
fn ecmwf_model_level_values_match_eccodes() {
    let messages: Vec<_> = read_messages(ECMWF_ML).collect();

    let cat = messages[0].data().unwrap();
    assert_eq!(cat.len(), 16 * 31);
    assert_close(cat[0], 0.5000008993995948, 1e-6, "cat first");
    assert_close(cat[cat.len() - 1], 0.9997872763527198, 1e-6, "cat last");
    assert_close(
        cat.iter().cloned().fold(f64::INFINITY, f64::min),
        8.99399594800343e-07,
        1e-6,
        "cat min",
    );
    assert_close(
        cat.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
        1.0000008993995948,
        1e-6,
        "cat max",
    );

    let lnsp = messages[1].data().unwrap();
    assert_eq!(lnsp.len(), 16 * 31);
    assert_close(lnsp[0], 11.52608871459961, 1e-5, "lnsp first");
    assert_close(lnsp[lnsp.len() - 1], 11.476189613342285, 1e-5, "lnsp last");
}

#[test]
fn ecmwf_model_levels_expose_pv() {
    for (message, meta) in read_messages(ECMWF_ML).zip(metadata()) {
        let pv = message.vertical_coordinate_values().unwrap().unwrap();
        assert_eq!(pv.len(), PV_LEN);
        for (actual, expected) in pv.iter().zip(PV_HEAD) {
            assert_close(*actual as f64, expected, 1e-6, "pv head");
        }
        assert_eq!(&pv[92..95], &[0.0, 0.0, 0.0]);
        assert_eq!(pv[PV_LEN - 1], 1.0);
        assert_eq!(
            meta.vertical_coordinate_values.as_deref(),
            Some(pv.as_slice())
        );
    }
}

#[test]
fn messages_without_coordinates_have_no_pv() {
    let data = std::fs::read("../test-data/aifs-single-t500.grib2").unwrap();
    let message = read_messages(&data).next().unwrap();
    assert_eq!(message.vertical_coordinate_values().unwrap(), None);
    let meta = MessageMetadata::try_from(&message).unwrap();
    assert_eq!(meta.vertical_coordinate_values, None);
}
