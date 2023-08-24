use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    message::{Message, MessageIterator},
    templates::product::tables::{
        FixedSurfaceType, GeneratingProcess, TimeUnit, TypeOfStatisticalProcessing,
    },
    utils::iter::projection::LatLngProjection,
};

#[derive(Clone, Debug)]
pub struct MessageMetadata {
    pub key: String,
    pub byte_offset: usize,
    pub message_size: usize,
    pub var: String,
    pub name: String,
    pub units: String,
    pub generating_process: GeneratingProcess,
    pub statistical_process: Option<TypeOfStatisticalProcessing>,
    pub time_unit: TimeUnit,
    pub time_increment_unit: Option<TimeUnit>,
    pub time_interval: u32,
    pub time_increment_interval: Option<u32>,
    pub first_fixed_surface_type: FixedSurfaceType,
    pub first_fixed_surface_value: Option<f64>,
    pub second_fixed_surface_type: FixedSurfaceType,
    pub second_fixed_surface_value: Option<f64>,
    pub discipline: String,
    pub category: String,
    pub data_compression: String,
    pub has_bitmap: bool,
    pub reference_date: DateTime<Utc>,
    pub forecast_date: DateTime<Utc>,
    pub forecast_end_date: Option<DateTime<Utc>>,
    pub proj: String,
    pub crs: String,
    pub is_regular_grid: bool,
    pub grid_shape: (usize, usize),
    pub projector: LatLngProjection,
}

impl MessageMetadata {
    pub fn data_point_count(&self) -> usize {
        self.grid_shape.0 * self.grid_shape.1
    }

    pub fn latlng(&self) -> (Vec<f64>, Vec<f64>) {
        self.projector.lat_lng()
    }

    pub fn as_idx(&self, index: usize) -> String {
        let formatted_date = self.reference_date.format("%Y%m%d%H").to_string();
        let level = if self.first_fixed_surface_type.is_single_level() {
            self.first_fixed_surface_type.name().into()
        } else {

            let level_value = if let Some(level_value) = self.first_fixed_surface_value {
                format!("{level_value:.0}")
            } else {
                "".to_string()
            };

            if self.first_fixed_surface_type.is_sequence_level() {
                format!(
                    "{level_value} in {}", self.first_fixed_surface_type.name()
                )
            } else {
                let level_unit = if self.first_fixed_surface_type.unit().len() > 0 {
                    format!(" {}", self.first_fixed_surface_type.unit())
                } else {
                    "".to_string()
                };

                format!(
                    "{level_value}{level_unit} {}", self.first_fixed_surface_type.name()
                )
            }
        };

        let statistical_process = if let Some(statistical_process) = self.statistical_process.as_ref() {
            format!("{} ", statistical_process.abbv())
        } else {
            "".to_string()
        };

        let time_offset = if let Some(time_increment_interval) = self.time_increment_interval {
            format!("{}-{} ", self.time_interval, time_increment_interval)
        } else {
            format!("{} ", self.time_interval)
        };

        let time_unit = format!("{} ", self.time_unit);

        format!(
            "{index}:{byte_offset}:d={formatted_date}:{var}:{level}:{time_offset}{time_unit}{statistical_process}{generating_process}:",
            index = index + 1,
            byte_offset = self.byte_offset,
            var = self.var,
            generating_process = self.generating_process.abbv(),
        )
    }
}

impl<'a> TryFrom<&Message<'a>> for MessageMetadata {
    type Error = String;

    fn try_from(message: &Message<'a>) -> Result<Self, Self::Error> {
        let (first_fixed_surface_type, first_fixed_surface_value) =
            message.first_fixed_surface()?;
        let (second_fixed_surface_type, second_fixed_surface_value) =
            message.second_fixed_surface()?;

        Ok(MessageMetadata {
            key: message.key()?,
            byte_offset: message.byte_offset(),
            message_size: message.len(),
            var: message.variable_abbrev()?,
            name: message.variable_name()?,
            units: message.unit()?,
            generating_process: message.generating_process()?,
            statistical_process: message.statistical_process_type()?,
            time_unit: message.time_unit()?,
            time_increment_unit: message.time_increment_unit()?,
            time_interval: message.time_interval()?,
            time_increment_interval: message.time_increment_interval()?,
            first_fixed_surface_type,
            first_fixed_surface_value,
            second_fixed_surface_type,
            second_fixed_surface_value,
            discipline: message.discipline()?.to_string(),
            category: message.category()?,
            data_compression: format!(
                "{}: {}",
                message.data_template_number().unwrap_or(99),
                message
                    .data_compression_type()
                    .unwrap_or("Unknown".to_string())
            ),
            has_bitmap: message.has_bitmap(),
            reference_date: message.reference_date()?,
            forecast_date: message.forecast_date()?,
            forecast_end_date: message.forecast_end_date()?,
            proj: message.proj_string()?,
            crs: message.crs()?,
            is_regular_grid: message.is_regular_grid()?,
            grid_shape: message.grid_dimensions()?,
            projector: message.latlng_projector()?,
        })
    }
}

pub fn scan_message_metadata<'a>(
    data: &'a [u8],
) -> HashMap<String, (usize, usize, MessageMetadata)> {
    let message_iter = MessageIterator::from_data(data, 0);

    message_iter
        .enumerate()
        .filter_map(|(index, m)| match MessageMetadata::try_from(&m) {
            Ok(mm) => Some(((&mm.key).clone(), (index, m.byte_offset(), mm))),
            Err(_) => None,
        })
        .collect()
}
