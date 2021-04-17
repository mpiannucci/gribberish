use std::ops::Range;

pub trait DataRepresentationTemplate<T> {
    fn bit_count_per_datapoint(&self) -> usize;
    fn decode_bits(&self, bits: Vec<u8>) -> Result<Vec<u8>, String>;
    fn scaled_value(&self, raw_value: T) -> T;
    fn unpack_range(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<T>, &'static str>;
    fn unpack_all(&self, bits: Vec<u8>) -> Result<Vec<T>, &'static str>;
}