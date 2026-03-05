pub trait GribSection {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn number(&self) -> u8;
}
