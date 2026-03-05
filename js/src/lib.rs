#![deny(clippy::all)]

use std::collections::HashMap;

use gribberish::{
    data_message::DataMessage,
    message::{read_message, read_messages, scan_messages},
    message_metadata::{scan_message_metadata, MessageMetadata},
};
use napi::bindgen_prelude::Uint8Array;
use napi_derive::napi;

#[napi(object)]
pub struct GridShape {
    pub rows: u32,
    pub cols: u32,
}

#[napi(object)]
pub struct LatLng {
    pub latitude: Vec<f64>,
    pub longitude: Vec<f64>,
}

#[napi]
pub struct GribMessage {
    inner: DataMessage,
}

#[napi]
impl GribMessage {
    #[napi(factory)]
    pub fn parse_from_buffer(buffer: Uint8Array, offset: u32) -> napi::Result<Self> {
        let buf = buffer.to_vec();
        let message = read_message(&buf, offset as usize)
            .ok_or_else(|| napi::Error::from_reason("Failed to read GRIB message at offset"))?;
        let message = DataMessage::try_from(&message)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(GribMessage { inner: message })
    }

    #[napi(getter)]
    pub fn key(&self) -> &str {
        self.inner.metadata.key.as_str()
    }

    #[napi(getter)]
    pub fn var_name(&self) -> &str {
        self.inner.metadata.name.as_str()
    }

    #[napi(getter)]
    pub fn var_abbrev(&self) -> &str {
        self.inner.metadata.var.as_str()
    }

    #[napi(getter)]
    pub fn units(&self) -> &str {
        self.inner.metadata.units.as_str()
    }

    #[napi(getter)]
    pub fn forecast_date(&self) -> chrono::DateTime<chrono::Utc> {
        self.inner.metadata.forecast_date
    }

    #[napi(getter)]
    pub fn reference_date(&self) -> chrono::DateTime<chrono::Utc> {
        self.inner.metadata.reference_date
    }

    #[napi(getter)]
    pub fn forecast_end_date(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.inner.metadata.forecast_end_date
    }

    #[napi(getter)]
    pub fn proj(&self) -> &str {
        self.inner.metadata.proj.as_str()
    }

    #[napi(getter)]
    pub fn crs(&self) -> &str {
        self.inner.metadata.crs.as_str()
    }

    #[napi(getter)]
    pub fn grid_shape(&self) -> GridShape {
        let (rows, cols) = self.inner.metadata.grid_shape;
        GridShape {
            rows: rows as u32,
            cols: cols as u32,
        }
    }

    #[napi(getter)]
    pub fn latlng(&self) -> LatLng {
        let (latitude, longitude) = self.inner.metadata.latlng();
        LatLng { latitude, longitude }
    }

    #[napi(getter)]
    pub fn is_regular_grid(&self) -> bool {
        self.inner.metadata.is_regular_grid
    }

    #[napi(getter)]
    pub fn has_bitmap(&self) -> bool {
        self.inner.metadata.has_bitmap
    }

    #[napi(getter)]
    pub fn perturbation_number(&self) -> Option<u32> {
        self.inner.metadata.perturbation_number.map(|n| n as u32)
    }

    #[napi(getter)]
    pub fn number_of_ensemble_members(&self) -> Option<u32> {
        self.inner.metadata.number_of_ensemble_members.map(|n| n as u32)
    }

    #[napi(getter)]
    pub fn data(&self) -> Vec<f64> {
        self.inner.data.clone()
    }
}

#[napi]
pub fn parse_messages_from_buffer(buffer: Uint8Array) -> Vec<GribMessage> {
    let buf = buffer.to_vec();
    read_messages(&buf)
        .filter_map(|gm| DataMessage::try_from(&gm).ok().map(|msg| GribMessage { inner: msg }))
        .collect()
}

#[napi]
pub struct GribMessageFactory {
    data: Vec<u8>,
    mapping: HashMap<String, (usize, usize)>,
}

#[napi]
impl GribMessageFactory {
    #[napi(factory)]
    pub fn from_buffer(buffer: Uint8Array) -> Self {
        let data = buffer.to_vec();
        let mapping = scan_messages(&data);
        GribMessageFactory { data, mapping }
    }

    #[napi(getter)]
    pub fn available_messages(&self) -> Vec<String> {
        self.mapping.keys().cloned().collect()
    }

    #[napi]
    pub fn get_message(&self, key: String) -> napi::Result<GribMessage> {
        let (_, offset) = self
            .mapping
            .get(&key)
            .ok_or_else(|| napi::Error::from_reason(format!("Message '{key}' not found")))?;
        let message = read_message(&self.data, *offset)
            .ok_or_else(|| napi::Error::from_reason("Failed to read GRIB message"))?;
        let message = DataMessage::try_from(&message)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(GribMessage { inner: message })
    }
}

#[napi]
pub struct GribMessageMetadataFactory {
    data: Vec<u8>,
    mapping: HashMap<String, (usize, usize, MessageMetadata)>,
}

#[napi]
impl GribMessageMetadataFactory {
    #[napi(factory)]
    pub fn from_buffer(buffer: Uint8Array) -> Self {
        let data = buffer.to_vec();
        let mapping = scan_message_metadata(&data);
        GribMessageMetadataFactory { data, mapping }
    }

    #[napi(getter)]
    pub fn available_messages(&self) -> Vec<String> {
        self.mapping.keys().cloned().collect()
    }

    #[napi]
    pub fn get_message(&self, key: String) -> napi::Result<GribMessage> {
        let (_, offset, metadata) = self
            .mapping
            .get(&key)
            .ok_or_else(|| napi::Error::from_reason(format!("Message '{key}' not found")))?;
        let message = read_message(&self.data, *offset)
            .ok_or_else(|| napi::Error::from_reason("Failed to read GRIB message"))?;
        let data_message = DataMessage::try_from((&message, metadata))
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        Ok(GribMessage { inner: data_message })
    }
}
