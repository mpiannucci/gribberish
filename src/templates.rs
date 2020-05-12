
enum TemplateType {
	Grid,
	Product,
	DataRepresentation,
	Data,
}

pub trait Template {

	fn data() -> &[u8];
}