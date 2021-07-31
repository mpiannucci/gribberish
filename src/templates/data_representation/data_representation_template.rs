use std::ops::Range;

pub trait DataRepresentationTemplate<T> {
    fn bit_count_per_datapoint(&self) -> usize;
    fn unpack(&self, bits: Vec<u8>, range: Option<Range<usize>>) -> Result<Vec<T>, String>;
    fn unpack_all(&self, bits: Vec<u8>) -> Result<Vec<T>, String> {
		self.unpack(bits, Option::None)
    }
}