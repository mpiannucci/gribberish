
pub trait GribSection {
	fn len(&self) -> usize;
	fn number(&self) -> u8;
}
