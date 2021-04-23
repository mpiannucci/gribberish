use std::ops::Range;

pub trait DataRepresentationTemplate<T> {
    fn bit_count_per_datapoint(&self) -> usize;
    fn unpack_range(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<T>, String>;
    fn unpack_all(&self, bits: Vec<u8>) -> Result<Vec<T>, String>;
}