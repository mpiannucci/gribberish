use bitvec::prelude::*;

pub trait DataRepresentationTemplate<T> {
    fn compression_type(&self) -> String;
    fn bit_count_per_datapoint(&self) -> usize;
    fn unpack(&self, bits: &BitSlice<u8, Msb0>) -> Result<Vec<T>, String>;
}