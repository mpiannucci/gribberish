use gribberish::message::read_messages;
use gribberish::message_metadata::{scan_message_metadata, MessageMetadata};
use gribberish::templates::product::tables::FixedSurfaceType;

// A 20 × 25 point sub-area (40N–38.1N, 100W–97.6W, 0.1°) cut with eccodes from two real
// messages of ECMWF's IFS model-level dissemination feed
// (sp3_um_ifs-ens-cf_od_oper_fc_20261012T120000Z_20261012T130000Z_1h, 12Z run, step 1):
// `lnsp` (paramId 152, 0/3/25) on hybrid level 1 and `cat` (paramId 260290, 0/19/29) on
// hybrid level 100. PDT 4.0 with NV = 276 (the IFS L137 `pv` array), CCSDS packing,
// centre ecmf. Only the grid was cut; the values, `pv` and all other metadata are the
// originals. Expected numbers are eccodes' decode of the cut file.
const ECMWF_ML: &[u8] = include_bytes!("../../test-data/ecmwf_ifs_ml_cat_lnsp_subarea.grib2");

const POINTS: usize = 20 * 25;
const PV_LEN: usize = 276;
const PV_HEAD: [f64; 3] = [0.0, 2.0003650188446045, 3.102241039276123];

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

    let lnsp = &meta[0];
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

    let cat = &meta[1];
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

    // scan_message_metadata silently drops messages whose metadata fails to parse.
    assert_eq!(scan_message_metadata(ECMWF_ML).len(), 2);
}

#[test]
fn ecmwf_model_level_values_match_eccodes() {
    let messages: Vec<_> = read_messages(ECMWF_ML).collect();
    assert_eq!(messages[0].grid_dimensions().unwrap(), (20, 25));

    let lnsp = messages[0].data().unwrap();
    assert_eq!(lnsp.len(), POINTS);
    assert_close(lnsp[0], 11.432563781738281, 1e-9, "lnsp first");
    assert_close(lnsp[123], 11.467964172363281, 1e-9, "lnsp [123]");
    assert_close(lnsp[POINTS - 1], 11.471138000488281, 1e-9, "lnsp last");
    let min = lnsp.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = lnsp.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert_close(min, 11.430610656738281, 1e-9, "lnsp min");
    assert_close(max, 11.475776672363281, 1e-9, "lnsp max");

    let cat = messages[1].data().unwrap();
    assert_eq!(cat.len(), POINTS);
    assert_close(cat[0], 0.01685124426148832, 1e-9, "cat first");
    assert_close(cat[123], 0.01949101476930082, 1e-9, "cat [123]");
    assert_close(cat[POINTS - 1], 0.014959154417738318, 1e-9, "cat last");
    let min = cat.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = cat.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert_close(min, 0.0022028067614883184, 1e-9, "cat min");
    assert_close(max, 0.15773564367555082, 1e-9, "cat max");
}

#[test]
fn ecmwf_model_levels_expose_pv() {
    for (message, meta) in read_messages(ECMWF_ML).zip(metadata()) {
        let pv = message.vertical_coordinate_values().unwrap().unwrap();
        assert_eq!(pv.len(), PV_LEN);
        for (actual, expected) in pv.iter().zip(PV_HEAD) {
            assert_close(*actual as f64, expected, 1e-6, "pv head");
        }
        // 138 `a` coefficients (Pa) then 138 `b` coefficients (0 at the top, 1 at the surface).
        let (a, b) = pv.split_at(PV_LEN / 2);
        assert_eq!(&b[..3], &[0.0, 0.0, 0.0]);
        assert_eq!(b[b.len() - 1], 1.0);
        assert!(b.windows(2).all(|w| w[0] <= w[1]));
        assert_close(
            a.iter().cloned().fold(f32::NEG_INFINITY, f32::max) as f64,
            20442.078125,
            1e-3,
            "a max",
        );
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
