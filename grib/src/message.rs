use crate::{
    sections::{indicator::Discipline, section::Section},
    templates::product::ProductTemplate,
};
use chrono::{DateTime, Utc};
use std::vec::Vec;

pub struct Message<'a> {
    pub sections: Vec<Section<'a>>,
}

pub struct MessageMetadata {
    pub discipline: Discipline,
    pub reference_date: DateTime<Utc>,
    pub forecast_date: DateTime<Utc>,
    pub variable_name: String,
    pub variable_abbreviation: String,
    pub region: ((f64, f64), (f64, f64)),
    pub location_grid: (usize, usize),
    pub location_resolution: (f64, f64),
    pub units: String,
}

impl<'a> Message<'a> {
    pub fn parse(data: &'a [u8], offset: usize) -> Result<Message<'a>, &'static str> {
        let mut sections: Vec<Section<'a>> = Vec::new();

        let mut current_offset = 0;
        loop {
            if let Some(section) = sections.last() {
                if let Section::End(_) = section {
                    break;
                }
            }

            let next_section = Section::from_data(data, offset + current_offset)?;
            current_offset += next_section.len();
            sections.push(next_section);
        }

        Ok(Message { sections })
    }

    pub fn parse_all(data: &'a [u8]) -> Vec<Message<'a>> {
        let mut messages = Vec::new();
        let mut offset: usize = 0;

        while offset < data.len() {
            if let Ok(message) = Message::parse(data, offset) {
                offset += message.len();
                messages.push(message);
            } else {
                break;
            }
        }

        messages
    }

    pub fn len(&self) -> usize {
        match self.sections.first() {
            Some(section) => match &section {
                Section::Indicator(indicator) => indicator.total_length() as usize,
                _ => 0,
            },
            None => 0,
        }
    }

    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    pub fn metadata(&self) -> Result<MessageMetadata, &'static str> {
        let discipline = match self.sections.first().unwrap() {
            Section::Indicator(indicator) => Ok(indicator.discipline()),
            _ => Err("Indicator section not found when reading discipline"),
        }?
        .clone();

        let reference_date = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Identification(identification) => Some(identification.reference_date()),
                _ => None,
            }),
            "Identification section not found when reading reference date"
        );

        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data"
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time"
        );
        let region = (grid_template.start(), grid_template.end());
        let location_grid = (grid_template.latitude_count(), grid_template.longitude_count());
        let location_resolution = (grid_template.latitude_resolution(), grid_template.longitude_resolution());

        let product_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::ProductDefinition(product_definition) => Some(product_definition),
                _ => None,
            }),
            "Product definition section not found when reading variable data"
        );

        let product_template = unwrap_or_return!(
            match product_definition.product_definition_template(discipline.clone() as u8) {
                ProductTemplate::HorizontalAnalysisForecast(template) => Some(template),
                _ => None,
            },
            "Only HorizontalAnalysisForecast templates are supported at this time"
        );

        let parameter = unwrap_or_return!(
            product_template.parameter(),
            "This Product and Parameter is currently not supported"
        );
        let forecast_date = product_template.forecast_datetime(reference_date);

        Ok(MessageMetadata {
            discipline,
            reference_date,
            forecast_date,
            variable_name: parameter.name,
            variable_abbreviation: parameter.abbrev,
            region,
            location_grid,
            location_resolution,
            units: parameter.unit,
        })
    }

    pub fn data(&self) -> Result<Vec<f64>, &'static str> {
        let data_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            "Data section not found when reading message data"
        );

        let raw_packed_data = data_section.raw_bit_data();

        let data_representation_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::DataRepresentation(data_representation_section) =>
                    Some(data_representation_section),
                _ => None,
            }),
            "Data representation section not found when reading message data"
        );

        let data_representation_template = unwrap_or_return!(
            data_representation_section.data_representation_template(),
            "Failed to unpack the data representation template"
        );

        let scaled_unpacked_data = data_representation_template
            .unpack_all(raw_packed_data)?
            .iter()
            .map(|v| data_representation_template.scaled_value(*v))
            .collect::<Vec<f64>>();

        let bitmap_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data"
        );

        let mapped_scaled_data = bitmap_section.map_data(scaled_unpacked_data);
        Ok(mapped_scaled_data)
    }

    pub fn data_locations(&self) -> Result<Vec<(f64, f64)>, &'static str> {
        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data"
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time"
        );

        Ok(grid_template.locations())
    }

    pub fn data_at_location(&self, location: &(f64, f64)) -> Result<f64, &'static str> {
        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data"
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time"
        );

        let location_index = grid_template.index_for_location(location.0, location.1)?;

        let data_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            "Data section not found when reading message data"
        );

        let data_representation_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::DataRepresentation(data_representation_section) =>
                    Some(data_representation_section),
                _ => None,
            }),
            "Data representation section not found when reading message data"
        );

        let data_representation_template = unwrap_or_return!(
            data_representation_section.data_representation_template(),
            "Failed to unpack the data representation template"
        );

        let bitmap_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data"
        );

        let data_index = unwrap_or_return!(
            bitmap_section.data_index(location_index), 
            "Invalid data index for the given location"
        );

        let raw_packed_data = data_section.raw_bit_data();
        let data = data_representation_template.unpack_range(raw_packed_data, data_index..data_index+1)?;

        Ok(data[0])
    }
}
