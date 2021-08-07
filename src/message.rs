use crate::{sections::{indicator::Discipline, section::Section}, templates::{product::ProductTemplate}};
use chrono::{DateTime, Utc};
use gribberish_types::Parameter;
use std::vec::Vec;

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
    pub data_template_number: u16,
    pub data_point_count: usize,
}

pub struct Message<'a> {
    pub sections: Vec<Section<'a>>,
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

    pub fn variable_names(messages: Vec<Message<'a>>) -> Vec<Option<String>> {
        Message::parameters(messages)
            .iter()
            .map(|p| {
                match p {
                    Some(p) => Some(p.name.clone()),
                    None => None,
                }
            })
            .collect()
    }

    pub fn variable_abbrevs(messages: Vec<Message<'a>>) -> Vec<Option<String>> {
        Message::parameters(messages)
            .iter()
            .map(|p| {
                match p {
                    Some(p) => Some(p.abbrev.clone()),
                    None => None,
                }
            })
            .collect()
    }

    pub fn units(messages: Vec<Message<'a>>) -> Vec<Option<String>> {
        Message::parameters(messages)
            .iter()
            .map(|p| {
                match p {
                    Some(p) => Some(p.unit.clone()),
                    None => None,
                }
            })
            .collect()
    }

    pub fn parameters(messages: Vec<Message<'a>>) -> Vec<Option<Parameter>> {
        messages
            .iter()
            .map(|m| m.parameter())
            .map(|r| {
                match r {
                    Ok(parameter) => Some(parameter),
                    Err(_) => None,
                }
            })
            .collect()
    }

    pub fn forecast_dates(messages: Vec<Message<'a>>) -> Vec<Option<DateTime<Utc>>> {
        messages
            .iter()
            .map(|m| m.forecast_date())
            .map(|r| {
                match r {
                    Ok(date) => Some(date),
                    Err(_) => None,
                }
            })
            .collect()
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

    pub fn discipline(&self) -> Result<Discipline, String> {
        match self.sections.first().unwrap() {
            Section::Indicator(indicator) => Ok(indicator.discipline()),
            _ => Err("Indicator section not found when reading discipline".into()),
        }.clone()
    }

    pub fn parameter(&self) -> Result<Parameter, String> {
        let discipline = self.discipline()?;

        let product_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::ProductDefinition(product_definition) => Some(product_definition),
                _ => None,
            }),
            "Product definition section not found when reading variable data".into()
        );

        let product_template = unwrap_or_return!(
            match product_definition.product_definition_template(discipline.clone() as u8) {
                ProductTemplate::HorizontalAnalysisForecast(template) => Some(template),
                _ => None,
            },
            "Only HorizontalAnalysisForecast templates are supported at this time".into()
        );

        let parameter = unwrap_or_return!(
            product_template.parameter(),
            "This Product and Parameter is currently not supported".into()
        );

        Ok(parameter)
    }

    pub fn variable_name(&self) -> Result<String, String> {
        let parameter = self.parameter()?;
        Ok(parameter.name)
    }

    pub fn variable_abbrev(&self) -> Result<String, String> {
        let parameter = self.parameter()?;
        Ok(parameter.abbrev)
    }

    pub fn reference_date(&self) -> Result<DateTime<Utc>, String> {
        let reference_date = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Identification(identification) => Some(identification.reference_date()),
                _ => None,
            }),
            "Identification section not found when reading reference date".into()
        );
        Ok(reference_date)
    }

    pub fn forecast_date(&self) -> Result<DateTime<Utc>, String> {
        let discipline = self.discipline()?;

        let product_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::ProductDefinition(product_definition) => Some(product_definition),
                _ => None,
            }),
            "Product definition section not found when reading variable data".into()
        );

        let product_template = unwrap_or_return!(
            match product_definition.product_definition_template(discipline.clone() as u8) {
                ProductTemplate::HorizontalAnalysisForecast(template) => Some(template),
                _ => None,
            },
            "Only HorizontalAnalysisForecast templates are supported at this time".into()
        );

        let reference_date = self.reference_date()?;
        Ok(product_template.forecast_datetime(reference_date))
    }

    pub fn metadata(&self) -> Result<MessageMetadata, String> {
        let discipline = self.discipline()?;

        let reference_date = self.reference_date()?;

        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );
        let region = (grid_template.start(), grid_template.end());
        let location_grid = (grid_template.latitude_count(), grid_template.longitude_count());
        let location_resolution = (grid_template.latitude_resolution(), grid_template.longitude_resolution());

        let parameter = self.parameter()?;
    
        let forecast_date = self.forecast_date()?;

        let data_representation = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            "Product definition section not found when reading variable data".into()
        );
        let data_template_number = data_representation.data_representation_template_number();
        let data_point_count = grid_definition.data_point_count();

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
            data_template_number,
            data_point_count
        })
    }

    pub fn data_index_for_location(&self, location: &(f64, f64)) -> Result<usize, String> {
        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        match grid_template.index_for_location(location.0, location.1) {
            Ok(res) => Ok(res), 
            Err(s) => Err(s.to_string())
        }
    }

    pub fn bitmap(&self) -> Result<Vec<bool>, String> {
        let bitmap_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data".into()
        );

        Ok(bitmap_section
            .bitmap()
            .iter()
            .map(|i| *i == 1u8)
            .collect())
    }

    pub fn data(&self) -> Result<Vec<f64>, String> {
        let data_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            "Data section not found when reading message data".into()
        );

        let raw_packed_data = data_section.raw_bit_data();

        let data_representation_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::DataRepresentation(data_representation_section) =>
                    Some(data_representation_section),
                _ => None,
            }),
            "Data representation section not found when reading message data".into()
        );

        let data_representation_template = unwrap_or_return!(
            data_representation_section.data_representation_template(),
            "Failed to unpack the data representation template".into()
        );

        let scaled_unpacked_data = data_representation_template
            .unpack(raw_packed_data, 0..data_representation_section.data_point_count())?;

        let bitmap_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data".into()
        );

        let mapped_scaled_data = bitmap_section.map_data(scaled_unpacked_data);
        Ok(mapped_scaled_data)
    }

    pub fn data_locations(&self) -> Result<Vec<(f64, f64)>, String> {
        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok(grid_template.locations())
    }

    pub fn data_at_location(&self, location: &(f64, f64)) -> Result<f64, String> {
        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        let location_index = grid_template.index_for_location(location.0, location.1)?;

        let data_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            "Data section not found when reading message data".into()
        );

        let data_representation_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::DataRepresentation(data_representation_section) =>
                    Some(data_representation_section),
                _ => None,
            }),
            "Data representation section not found when reading message data".into()
        );

        let data_representation_template = unwrap_or_return!(
            data_representation_section.data_representation_template(),
            "Failed to unpack the data representation template".into()
        );

        let bitmap_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data".into()
        );

        let data_index = unwrap_or_return!(
            bitmap_section.data_index(location_index), 
            format!("No data available at index {}", location_index).into()
        );

        let raw_packed_data = data_section.raw_bit_data();
        let data = data_representation_template.unpack(raw_packed_data, data_index..data_index+1)?;

        Ok(data[0])
    }

    pub fn data_in_region(&self, top_left: &(f64, f64), bottom_right: &(f64, f64)) -> Result<Vec<f64>, String> {
        let top_left_index = self.data_index_for_location(top_left)?;
        let bottom_right_index = self.data_index_for_location(bottom_right)?;

        let grid_definition = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        let data_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            "Data section not found when reading message data".into()
        );

        let data_representation_section = unwrap_or_return!(
            self.sections.iter().find_map(|s| match s {
                Section::DataRepresentation(data_representation_section) =>
                    Some(data_representation_section),
                _ => None,
            }),
            "Data representation section not found when reading message data".into()
        );

        let data_representation_template = unwrap_or_return!(
            data_representation_section.data_representation_template(),
            "Failed to unpack the data representation template".into()
        );

        let raw_packed_data = data_section.raw_bit_data();
        let data = data_representation_template.unpack(raw_packed_data, top_left_index..bottom_right_index)?;

        Ok(data)
    }
}
