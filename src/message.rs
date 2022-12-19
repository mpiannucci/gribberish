use crate::templates::grid_definition::GridDefinitionTemplate;
use crate::{
    sections::{indicator::Discipline, section::Section, section::SectionIterator},
    templates::product::ProductTemplate,
};
use chrono::{DateTime, Utc};
use gribberish_types::Parameter;
use std::collections::HashMap;
use std::vec::Vec;

pub fn map_messages<'a>(data: &'a [u8]) -> HashMap<String, (usize, usize)> {
    let message_iter = MessageIterator::from_data(data, 0);

    message_iter
        .enumerate()
        .map(
            |(index, m)| match m.variable_abbrev().or(m.parameter_index()) {
                Ok(var) => (var, (index, m.byte_offset())),
                Err(_) => ("unknown".into(), (index, m.byte_offset())),
            },
        )
        .collect()
}

pub fn read_messages<'a>(data: &'a [u8]) -> MessageIterator<'a> {
    MessageIterator { data, offset: 0 }
}

pub fn read_message<'a>(data: &'a [u8], offset: usize) -> Option<Message<'a>> {
    Message::from_data(data, offset)
}

pub struct MessageIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> MessageIterator<'a> {
    pub fn from_data(data: &'a [u8], offset: usize) -> Self {
        MessageIterator { data, offset }
    }

    pub fn current_offset(&self) -> usize {
        self.offset
    }
}

impl<'a> Iterator for MessageIterator<'a> {
    type Item = Message<'a>;

    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        match Message::from_data(&self.data, self.offset) {
            Some(m) => {
                self.offset += m.len();
                Some(m)
            }
            None => None,
        }
    }
}

pub struct Message<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> Message<'a> {
    pub fn from_data(data: &'a [u8], offset: usize) -> Option<Message> {
        let mut sections = SectionIterator { data: data, offset };

        match sections.next() {
            Some(Section::Indicator(i)) => Some(Message {
                data: &data,
                offset: offset,
            }),
            _ => None,
        }
    }

    pub fn byte_data(&self) -> &'a [u8] {
        self.data
    }

    pub fn byte_offset(&self) -> usize {
        self.offset
    }

    pub fn sections(&self) -> SectionIterator {
        SectionIterator {
            data: self.data,
            offset: self.offset,
        }
    }

    pub fn variable_names(messages: Vec<Message>) -> Vec<Option<String>> {
        Message::parameters(messages)
            .iter()
            .map(|p| match p {
                Some(p) => Some(p.name.clone()),
                None => None,
            })
            .collect()
    }

    pub fn variable_abbrevs(messages: Vec<Message>) -> Vec<Option<String>> {
        Message::parameters(messages)
            .iter()
            .map(|p| match p {
                Some(p) => Some(p.abbrev.clone()),
                None => None,
            })
            .collect()
    }

    pub fn units(messages: Vec<Message>) -> Vec<Option<String>> {
        Message::parameters(messages)
            .iter()
            .map(|p| match p {
                Some(p) => Some(p.unit.clone()),
                None => None,
            })
            .collect()
    }

    pub fn parameters(messages: Vec<Message>) -> Vec<Option<Parameter>> {
        messages
            .iter()
            .map(|m| m.parameter())
            .map(|r| match r {
                Ok(parameter) => Some(parameter),
                Err(_) => None,
            })
            .collect()
    }

    pub fn forecast_dates(messages: Vec<Message>) -> Vec<Option<DateTime<Utc>>> {
        messages
            .iter()
            .map(|m| m.forecast_date())
            .map(|r| match r {
                Ok(date) => Some(date),
                Err(_) => None,
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        match self.sections().next() {
            Some(Section::Indicator(i)) => i.total_length() as usize,
            Some(_) => 0,
            None => 0,
        }
    }

    pub fn section_count(&self) -> usize {
        self.sections().count()
    }

    pub fn discipline(&self) -> Result<Discipline, String> {
        match self.sections().next().unwrap() {
            Section::Indicator(indicator) => Ok(indicator.discipline()),
            _ => Err("Indicator section not found when reading discipline".into()),
        }
        .clone()
    }

    pub fn parameter_index(&self) -> Result<String, String> {
        let mut sections = self.sections();

        let discipline = unwrap_or_return!(
            sections.find_map(|s| match s {
                Section::Indicator(indicator) => Some(indicator.discipline_value()),
                _ => None,
            }),
            "Failed to read discipline value from indicator section".into()
        );

        let product_definition = unwrap_or_return!(
            sections.find_map(|s| match s {
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

        let category = product_template.category_value();
        let parameter = product_template.parameter_value();

        Ok(format!("({}, {}, {})", discipline, category, parameter))
    }

    pub fn parameter(&self) -> Result<Parameter, String> {
        let discipline = self.discipline()?;

        let product_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
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
            format!(
                "This Product and Parameter is currently not supported: ({}, {})",
                product_template.category_value(),
                product_template.parameter_value()
            )
            .into()
        );

        Ok(parameter)
    }

    pub fn category(&self) -> Result<String, String> {
        let discipline = self.discipline()?;

        let product_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
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

        Ok(product_template.category().to_owned())
    }

    pub fn variable_name(&self) -> Result<String, String> {
        let parameter = self.parameter()?;
        Ok(parameter.name)
    }

    pub fn variable_abbrev(&self) -> Result<String, String> {
        let parameter = self.parameter()?;
        Ok(parameter.abbrev)
    }

    pub fn unit(&self) -> Result<String, String> {
        let parameter = self.parameter()?;
        Ok(parameter.unit)
    }

    pub fn reference_date(&self) -> Result<DateTime<Utc>, String> {
        let reference_date = unwrap_or_return!(
            self.sections().find_map(|s| match s {
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
            self.sections().find_map(|s| match s {
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

    pub fn array_index(&self) -> Result<Option<usize>, String> {
        let discipline = self.discipline()?;

        let product_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
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

        Ok(product_template.array_index())
    }

    pub fn proj_string(&self) -> Result<String, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok(grid_template.proj_string())
    }

    pub fn crs(&self) -> Result<String, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok(grid_template.crs())
    }

    pub fn location_region(&self) -> Result<((f64, f64), (f64, f64)), String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok((grid_template.start(), grid_template.end()))
    }

    pub fn location_grid_dimensions(&self) -> Result<(usize, usize), String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok((
            grid_template.latitude_count(),
            grid_template.longitude_count(),
        ))
    }

    pub fn location_resolution(&self) -> Result<(f64, f64), String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok((
            grid_template.latitude_resolution(),
            grid_template.longitude_resolution(),
        ))
    }

    pub fn locations(&self) -> Result<Vec<(f64, f64)>, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
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

    pub fn latitudes(&self) -> Result<Vec<f64>, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok(grid_template.latitudes())
    }

    pub fn longitudes(&self) -> Result<Vec<f64>, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok(grid_template.longitudes())
    }

    pub fn data_template_number(&self) -> Result<u16, String> {
        let data_representation = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            "Product definition section not found when reading variable data".into()
        );

        Ok(data_representation.data_representation_template_number())
    }

    pub fn data_compression_type(&self) -> Result<String, String> {
        let data_representation = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            "Product definition section not found when reading variable data".into()
        );

        let data_representation_template = unwrap_or_return!(
            data_representation.data_representation_template(),
            "Failed to unpack the data representation template".into()
        );

        Ok(data_representation_template.compression_type())
    }

    pub fn data_point_count(&self) -> Result<usize, String> {
        let data_representation = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            "Product definition section not found when reading variable data".into()
        );

        Ok(data_representation.data_point_count())
    }

    pub fn has_bitmap(&self) -> bool {
        let bitmap_section = self.sections().find_map(|s| match s {
            Section::Bitmap(bitmap_section) => Some(bitmap_section),
            _ => None,
        });

        match bitmap_section {
            Some(b) => b.has_bitmap(),
            None => false,
        }
    }

    pub fn bitmap(&self) -> Result<Vec<bool>, String> {
        let bitmap_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data".into()
        );

        Ok(bitmap_section.bitmap().iter().map(|i| *i == 1u8).collect())
    }

    pub fn data(&self) -> Result<Vec<f64>, String> {
        let data_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            "Data section not found when reading message data".into()
        );

        let raw_packed_data = data_section.raw_bit_data();

        let data_representation_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
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

        let scaled_unpacked_data = data_representation_template.unpack(
            raw_packed_data,
            0..data_representation_section.data_point_count(),
        )?;

        let bitmap_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            "Bitmap section not found when reading message data".into()
        );

        if bitmap_section.has_bitmap() {
            let mapped_scaled_data = bitmap_section.map_data(scaled_unpacked_data);
            Ok(mapped_scaled_data)
        } else {
            Ok(scaled_unpacked_data)
        }
    }

    pub fn data_grid(&self) -> Result<Vec<Vec<f64>>, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        let data = self.data()?;
        let j_count = grid_template.longitude_count();

        let data_grid = grid_template
            .zerod_location_grid()
            .iter()
            .enumerate()
            .map(|i| {
                i.1.iter()
                    .enumerate()
                    .map(|j| {
                        let index = (i.0 * j_count) + j.0;
                        data[index]
                    })
                    .collect()
            })
            .collect();

        Ok(data_grid)
    }

    pub fn location_grid(&self) -> Result<Vec<Vec<Vec<f64>>>, String> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            "Grid definition section not found when reading variable data".into()
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            "Only latitude longitude templates supported at this time".into()
        );

        Ok(grid_template.location_grid())
    }
}
