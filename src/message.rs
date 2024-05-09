use crate::error::GribberishError;
use crate::sections::{indicator::Discipline, section::Section, section::SectionIterator};
use crate::templates::grid_definition::GridDefinitionTemplate;
use crate::templates::product::product_template::ProductTemplate;
use crate::templates::product::tables::{
    DerivedForecastType, FixedSurfaceType, GeneratingProcess, TimeUnit, TypeOfStatisticalProcessing,
};
use crate::utils::iter::projection::LatLngProjection;
use bitvec::view::BitView;
use chrono::{DateTime, Utc};
use gribberish_types::Parameter;
use std::collections::HashMap;
use std::vec::Vec;

pub fn scan_messages<'a>(data: &'a [u8]) -> HashMap<String, (usize, usize)> {
    let message_iter = MessageIterator::from_data(data, 0);

    message_iter
        .enumerate()
        .map(|(index, m)| match m.key() {
            Ok(var) => (var, (index, m.byte_offset())),
            Err(_) => ("unknown".into(), (index, m.byte_offset())),
        })
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
            Some(Section::Indicator(_)) => Some(Message {
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

    pub fn key(&self) -> Result<String, GribberishError> {
        let time = self
            .forecast_date()
            .map(|t| format!(":{}", t.format("%Y%m%d%H%M")))
            .unwrap_or("".into())
            .to_string();
        let var = self.variable_abbrev()?;
        let generating_process = self.generating_process()?.to_string();
        let statistical_process = self
            .statistical_process_type()
            .unwrap_or(None)
            .map_or("".to_string(), |s| format!("{s} "));
        let first_fixed_surface = self.first_fixed_surface()?;
        let first_level = if first_fixed_surface.0 == FixedSurfaceType::Missing {
            "".into()
        } else {
            let level_value = if let Some(value) = first_fixed_surface.1 {
                format!("{:.0}", value)
            } else {
                "".into()
            };

            format!(
                ":{level_value} in {}",
                Parameter::from(first_fixed_surface.0).name
            )
        };

        let second_fixed_surface = self.second_fixed_surface()?;
        let second_level = if second_fixed_surface.0 == FixedSurfaceType::Missing {
            "".into()
        } else {
            let level_value = if let Some(value) = second_fixed_surface.1 {
                format!("{:.0}", value)
            } else {
                "".into()
            };

            format!(
                ":{level_value} in {}",
                Parameter::from(second_fixed_surface.0).name
            )
        };

        Ok(format!(
            "{var}{time}{first_level}{second_level}:{statistical_process}{generating_process}"
        ))
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

    pub fn discipline(&self) -> Result<Discipline, GribberishError> {
        match self.sections().next().unwrap() {
            Section::Indicator(indicator) => Ok(indicator.discipline()),
            _ => Err(GribberishError::MessageError(
                "Indicator section not found when reading discipline".into(),
            )),
        }
        .clone()
    }

    pub fn product_template_id(&self) -> Result<u16, GribberishError> {
        let mut sections = self.sections();

        let product_definition = unwrap_or_return!(
            sections.find_map(|s| match s {
                Section::ProductDefinition(product_definition) => Some(product_definition),
                _ => None,
            }),
            GribberishError::MessageError(
                "Product definition section not found when reading variable data".into()
            )
        );

        Ok(product_definition.product_definition_template_number())
    }

    pub fn product_template(&self) -> Result<Box<dyn ProductTemplate>, GribberishError> {
        let mut sections = self.sections();

        let discipline = unwrap_or_return!(
            sections.find_map(|s| match s {
                Section::Indicator(indicator) => Some(indicator.discipline_value()),
                _ => None,
            }),
            GribberishError::MessageError(
                "Failed to read discipline value from indicator section".into()
            )
        );

        let product_definition = unwrap_or_return!(
            sections.find_map(|s| match s {
                Section::ProductDefinition(product_definition) => Some(product_definition),
                _ => None,
            }),
            GribberishError::MessageError(
                "Product definition section not found when reading variable data".into()
            )
        );

        let product_template = unwrap_or_return!(
            product_definition.product_definition_template(discipline),
            GribberishError::MessageError(
                "Only HorizontalAnalysisForecast templates are supported at this time".into()
            )
        );

        Ok(product_template)
    }

    pub fn grid_template(&self) -> Result<Box<dyn GridDefinitionTemplate>, GribberishError> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            GribberishError::MessageError("Grid definition section not found when reading variable data".into())
        );

        let grid_template = unwrap_or_return!(
            grid_definition.grid_definition_template(),
            GribberishError::MessageError("Only latitude longitude templates supported at this time".into())
        );

        Ok(grid_template)
    }

    pub fn parameter_index(&self) -> Result<String, GribberishError> {
        let discipline = self.discipline()?;
        let product_template = self.product_template()?;
        let category = product_template.category_value();
        let parameter = product_template.parameter_value();

        Ok(format!("({}, {}, {})", discipline, category, parameter))
    }

    pub fn parameter(&self) -> Result<Parameter, GribberishError> {
        let product_template = self.product_template()?;

        let parameter = unwrap_or_return!(
            product_template.parameter(),
            GribberishError::MessageError(
                format!(
                    "This Product and Parameter is currently not supported: ({}, {})",
                    product_template.category_value(),
                    product_template.parameter_value()
                )
                .into()
            )
        );

        Ok(parameter)
    }

    pub fn category(&self) -> Result<String, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.category().to_owned())
    }

    pub fn variable_name(&self) -> Result<String, GribberishError> {
        let parameter = self.parameter()?;
        Ok(parameter.name)
    }

    pub fn variable_abbrev(&self) -> Result<String, GribberishError> {
        let parameter = self.parameter()?;
        Ok(parameter.abbrev)
    }

    pub fn unit(&self) -> Result<String, GribberishError> {
        let parameter = self.parameter()?;
        Ok(parameter.unit)
    }

    pub fn reference_date(&self) -> Result<DateTime<Utc>, GribberishError> {
        let reference_date = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::Identification(identification) => Some(identification.reference_date()),
                _ => None,
            }),
            GribberishError::MessageError(
                "Identification section not found when reading reference date".into()
            )
        );
        Ok(reference_date)
    }

    pub fn generating_process(&self) -> Result<GeneratingProcess, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.generating_process())
    }

    pub fn derived_forecast_type(&self) -> Result<Option<DerivedForecastType>, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.derived_forecast_type())
    }

    pub fn statistical_process_type(
        &self,
    ) -> Result<Option<TypeOfStatisticalProcessing>, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.statistical_process_type())
    }

    pub fn forecast_date(&self) -> Result<DateTime<Utc>, GribberishError> {
        let product_template = self.product_template()?;
        let reference_date = self.reference_date()?;
        Ok(product_template.forecast_datetime(reference_date))
    }

    pub fn forecast_end_date(&self) -> Result<Option<DateTime<Utc>>, GribberishError> {
        let product_template = self.product_template()?;
        let reference_date = self.reference_date()?;
        Ok(product_template.forecast_end_datetime(reference_date))
    }

    pub fn time_unit(&self) -> Result<TimeUnit, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.time_unit())
    }

    pub fn time_increment_unit(&self) -> Result<Option<TimeUnit>, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.time_increment_unit())
    }

    pub fn time_interval(&self) -> Result<u32, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.time_interval())
    }

    pub fn time_increment_interval(&self) -> Result<Option<u32>, GribberishError> {
        let product_template = self.product_template()?;
        Ok(product_template.time_increment_interval())
    }

    pub fn first_fixed_surface(&self) -> Result<(FixedSurfaceType, Option<f64>), GribberishError> {
        let product_template = self.product_template()?;
        let surface_type = product_template.first_fixed_surface_type();
        let surface_value = product_template.first_fixed_surface_value();
        Ok((surface_type, surface_value))
    }

    pub fn second_fixed_surface(&self) -> Result<(FixedSurfaceType, Option<f64>), GribberishError> {
        let product_template = self.product_template()?;
        let surface_type = product_template.second_fixed_surface_type();
        let surface_value = product_template.second_fixed_surface_value();
        Ok((surface_type, surface_value))
    }

    pub fn proj_string(&self) -> Result<String, GribberishError> {
        let grid_template = self.grid_template()?;
        Ok(grid_template.proj_string())
    }

    pub fn grid_template_id(&self) -> Result<u16, GribberishError> {
        let grid_definition = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::GridDefinition(grid_definition) => Some(grid_definition),
                _ => None,
            }),
            GribberishError::MessageError(
                "Grid definition section not found when reading variable data".into()
            )
        );

        Ok(grid_definition.grid_definition_template_number())
    }

    pub fn is_regular_grid(&self) -> Result<bool, GribberishError> {
        let grid_template = self.grid_template()?;
        Ok(grid_template.is_regular_grid())
    }

    pub fn crs(&self) -> Result<String, GribberishError> {
        let grid_template = self.grid_template()?;
        Ok(grid_template.crs())
    }

    pub fn grid_dimensions(&self) -> Result<(usize, usize), GribberishError> {
        let grid_template = self.grid_template()?;
        Ok((grid_template.y_count(), grid_template.x_count()))
    }

    pub fn latlng_projector(&self) -> Result<LatLngProjection, GribberishError> {
        let grid_template = self.grid_template()?;
        Ok(grid_template.projector())
    }

    pub fn data_template_number(&self) -> Result<u16, GribberishError> {
        let data_representation = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            GribberishError::MessageError(
                "Product definition section not found when reading variable data".into()
            )
        );

        Ok(data_representation.data_representation_template_number())
    }

    pub fn data_compression_type(&self) -> Result<String, GribberishError> {
        let data_representation = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            GribberishError::MessageError(
                "Product definition section not found when reading variable data".into()
            )
        );

        let data_representation_template = unwrap_or_return!(
            data_representation.data_representation_template(),
            GribberishError::MessageError(
                "Failed to unpack the data representation template".into()
            )
        );

        Ok(data_representation_template.compression_type())
    }

    pub fn data_point_count(&self) -> Result<usize, GribberishError> {
        let data_representation = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation) => Some(data_representation),
                _ => None,
            }),
            GribberishError::MessageError(
                "Product definition section not found when reading variable data".into()
            )
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

    pub fn data(&self) -> Result<Vec<f64>, GribberishError> {
        let data_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::Data(data_section) => Some(data_section),
                _ => None,
            }),
            GribberishError::MessageError(
                "Data section not found when reading message data".into()
            )
        );

        let raw_packed_data = data_section.raw_data_array().view_bits();

        let data_representation_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::DataRepresentation(data_representation_section) =>
                    Some(data_representation_section),
                _ => None,
            }),
            GribberishError::MessageError(
                "Data representation section not found when reading message data".into()
            )
        );

        let data_representation_template = unwrap_or_return!(
            data_representation_section.data_representation_template(),
            GribberishError::MessageError(
                "Failed to unpack the data representation template".into()
            )
        );

        let scaled_unpacked_data = data_representation_template.unpack(raw_packed_data)?;

        let bitmap_section = unwrap_or_return!(
            self.sections().find_map(|s| match s {
                Section::Bitmap(bitmap_section) => Some(bitmap_section),
                _ => None,
            }),
            GribberishError::MessageError(
                "Bitmap section not found when reading message data".into()
            )
        );

        let mut data = if bitmap_section.has_bitmap() {
            let mapped_scaled_data = bitmap_section.map_data(scaled_unpacked_data);
            mapped_scaled_data
        } else {
            scaled_unpacked_data
        };

        let shape = self.grid_dimensions()?;
        let count = shape.0 * shape.1;
        data.resize(count, 0.0);
        Ok(data)
    }
}
