use chrono::prelude::*;
use crate::sections::indicator::Discipline;
use crate::sections::grid_definition::GridDefinitionSection;
use crate::sections::product_definition::ProductDefinitionSection;
use crate::sections::bitmap::BitmapSection;
use crate::sections::data::DataSection;

pub struct Field<'a> {
    pub discipline: Discipline,
    pub reference_date: DateTime<Utc>,
    pub grid_definition: &'a GridDefinitionSection<'a>,
    pub product_definition: &'a ProductDefinitionSection<'a>,
    pub bitmap: &'a BitmapSection<'a>,
    pub data: &'a DataSection<'a>,
}
