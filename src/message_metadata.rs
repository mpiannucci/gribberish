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
    pub message_size: usize,
    pub var: String,
    pub name: String,
    pub units: String,
    pub generating_process: GeneratingProcess,
    pub statistical_process: Option<TypeOfStatisticalProcessing>,
    pub time_unit: TimeUnit,
    pub first_fixed_surface_type: FixedSurfaceType,
    pub first_fixed_surface_value: Option<f64>,
    pub second_fixed_surface_type: FixedSurfaceType,
    pub second_fixed_surface_value: Option<f64>,
    pub discipline: String,
    pub category: String,
    pub data_compression: String,
    pub has_bitmap: bool,
    pub forecast_date: DateTime<Utc>,
    pub reference_date: DateTime<Utc>,
    pub time_interval_end: Option<DateTime<Utc>>,
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
            message_size: message.len(),
            var: message.variable_abbrev()?,
            name: message.variable_name()?,
            units: message.unit()?,
            generating_process: message.generating_process()?,
            statistical_process: message.statistical_process_type()?,
            time_unit: message.time_unit()?,
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
            time_interval_end: message.time_interval_end()?,
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
