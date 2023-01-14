use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::{
    message::{Message, MessageIterator},
    templates::product::tables::{FixedSurfaceType, GeneratingProcess},
};

#[derive(Clone, Debug)]
pub struct MessageMetadata {
    pub key: String,
    pub var: String,
    pub name: String,
    pub units: String,
    pub generating_process: GeneratingProcess,
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
    pub proj: String,
    pub crs: String,
    pub bbox: (f64, f64, f64, f64),
    pub is_regular_grid: bool,
    pub grid_shape: (usize, usize),
    pub latitude: Vec<f64>,
    pub longitude: Vec<f64>,
}

impl MessageMetadata {
    pub fn data_point_count(&self) -> usize {
        self.grid_shape.0 * self.grid_shape.1
    }

    pub fn flattened_coords(&self) -> Vec<(f64, f64)> {
        let (_, cols) = self.grid_shape;
        (0..self.data_point_count())
            .map(|i| {
                let lat_i = i / cols;
                let lng_i = i % cols;

                (self.latitude[lat_i], self.longitude[lng_i])
            })
            .collect()
    }
}

impl<'a> TryFrom<&Message<'a>> for MessageMetadata {
    type Error = String;

    fn try_from(message: &Message<'a>) -> Result<Self, Self::Error> {
        let (first_fixed_surface_type, first_fixed_surface_value) =
            message.first_fixed_surface()?;
        let (second_fixed_surface_type, second_fixed_surface_value) =
            message.second_fixed_surface()?;
            let (latitudes, longitudes) = message.latitude_longitude_arrays()?;

        Ok(MessageMetadata {
            key: message.key()?,
            var: message.variable_abbrev()?,
            name: message.variable_name()?,
            units: message.unit()?,
            generating_process: message.generating_process()?,
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
            forecast_date: message.forecast_date()?,
            reference_date: message.reference_date()?,
            proj: message.proj_string()?,
            crs: message.crs()?,
            bbox: message.location_bbox()?,
            is_regular_grid: message.is_regular_grid()?,
            grid_shape: message.grid_dimensions()?,
            latitude: latitudes,
            longitude: longitudes,
        })
    }
}

pub fn scan_message_metadata<'a>(data: &'a [u8]) -> HashMap<String, (usize, usize, Result<MessageMetadata, String>)> {
    let message_iter = MessageIterator::from_data(data, 0);

    message_iter
        .enumerate()
        .map(
            |(index, m)| match m.key() {
                Ok(var) => (var, (index, m.byte_offset(), MessageMetadata::try_from(&m))),
                Err(_) => ("unknown".into(), (index, m.byte_offset(), Err("Could not unpack metadata".into()))),
            },
        )
        .collect()
}
