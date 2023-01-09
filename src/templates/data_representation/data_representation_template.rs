
pub trait DataRepresentationTemplate<T> {
    fn compression_type(&self) -> String;
    fn bit_count_per_datapoint(&self) -> usize;
    fn unpack(&self, bits: Vec<u8>) -> Result<Vec<T>, String>;
}