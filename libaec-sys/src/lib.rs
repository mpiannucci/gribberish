#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_aec_stream_size() {
        // Basic sanity check that the bindings work
        let stream_size = mem::size_of::<aec_stream>();
        assert!(stream_size > 0);
    }

    #[test]
    fn test_aec_constants() {
        // Test that important constants are defined
        assert_eq!(AEC_OK, 0);
        assert!(AEC_CONF_ERROR as u32 != AEC_OK);
        assert!(AEC_STREAM_ERROR as u32 != AEC_OK);
        assert!(AEC_DATA_ERROR as u32 != AEC_OK);
        assert!(AEC_MEM_ERROR as u32 != AEC_OK);
    }
}
