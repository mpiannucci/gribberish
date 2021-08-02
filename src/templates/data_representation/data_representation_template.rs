use std::ops::Range;

pub trait DataRepresentationTemplate<T> {
    fn bit_count_per_datapoint(&self) -> usize;
    fn unpack(&self, bits: Vec<u8>, range: Range<usize>) -> Result<Vec<T>, String>;
}