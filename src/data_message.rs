use std::convert::TryFrom;
use crate::message::Message;
use chrono::{DateTime, Utc};

pub struct DataMessage {
    pub var: String,
    pub name: String,
    pub units: String,
    pub array_index: Option<usize>,
    pub discipline: String, 
    pub category: String, 
    pub data_compression: String,
    pub has_bitmap: bool,
    pub forecast_date: DateTime<Utc>,
    pub reference_date: DateTime<Utc>,
    pub proj: String,
    pub crs: String,
    pub bbox: (f64, f64, f64, f64),
    pub grid_resolution: (f64, f64),
    pub latitude: Vec<f64>,
    pub longitude: Vec<f64>,
    pub data: Vec<Vec<f64>>
}

impl DataMessage {
    pub fn flattened_data(&self) -> Vec<f64> {
        self.data.clone().into_iter().flatten().collect()
    }

    pub fn grid_shape(&self) -> (usize, usize) {
        (self.latitude.len(), self.longitude.len())
    }

    pub fn data_point_count(&self) -> usize {
        self.latitude.len() * self.longitude.len()
    }

    pub fn flattened_coords(&self) -> Vec<(f64, f64)> {
        let (_, cols) = self.grid_shape();
        (0..self.data_point_count())
            .map(|i| {
                let lat_i = i / cols;
                let lng_i = i % cols;

                (self.latitude[lat_i], self.longitude[lng_i])
            })
            .collect()
    }
}

impl <'a> TryFrom<Message<'a>> for DataMessage {
    type Error = String;

    fn try_from(message: Message) -> std::result::Result<Self, <Self as std::convert::TryFrom<Message>>::Error> { 
        Ok(DataMessage {
            var: message.variable_abbrev()?, 
            name: message.variable_name()?, 
            units: message.unit()?, 
            array_index: message.array_index()?,
            discipline: message.discipline()?.to_string(),
            category: message.category()?,
            data_compression: format!("{}: {}", message.data_template_number().unwrap_or(99), message.data_compression_type().unwrap_or("Unknown".to_string())),
            has_bitmap: message.has_bitmap(),
            forecast_date: message.forecast_date()?, 
            reference_date: message.reference_date()?, 
            proj: message.proj_string()?,
            crs: message.crs()?,
            bbox: message.location_bbox()?,
            grid_resolution: message.location_resolution()?,
            latitude: message.latitudes()?, 
            longitude: message.longitudes()?, 
            data: message.data_grid()?,
        })
    }
}
