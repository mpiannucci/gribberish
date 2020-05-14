use grib_data_derive::{DisplayDescription, FromValue};

#[repr(u8)]
#[derive(Eq, PartialEq, Debug, DisplayDescription, FromValue)]
pub enum TemplateType {
    Grid = 3,
    Product = 4,
    DataRepresentation = 5,
    Data = 6,
}

pub trait Template {
    fn template_type(&self) -> TemplateType;

    fn template_number(&self) -> u16;

    fn data(&self) -> &[u8];
}