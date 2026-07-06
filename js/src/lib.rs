#![deny(clippy::all)]

use std::collections::HashMap;

use gribberish::{
  adjust_latitude_values as adjust_latitude_values_core,
  adjust_longitude_values as adjust_longitude_values_core,
  data_message::DataMessage,
  index::parse_index,
  message::{read_message, read_messages, scan_messages},
  message_metadata::{MessageMetadata, scan_message_metadata},
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
    let message =
      DataMessage::try_from(&message).map_err(|e| napi::Error::from_reason(e.to_string()))?;
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
    LatLng {
      latitude,
      longitude,
    }
  }

  /// Like the `latlng` getter: `adjustLongitudeRange` wraps an eligible
  /// near-global grid's longitudes from `[0, 360)` to a monotonic `[-180, 180)`;
  /// `northUp` reorders rows so the 0th row is the northern-most. Pair with
  /// `dataAdjusted` using the same flags so the values stay aligned. Each flag
  /// is a no-op when the grid is ineligible.
  #[napi]
  pub fn latlng_adjusted(&self, adjust_longitude_range: bool, north_up: Option<bool>) -> LatLng {
    let (latitude, longitude) = self
      .inner
      .metadata
      .latlng_adjusted(adjust_longitude_range, north_up.unwrap_or(false));
    LatLng {
      latitude,
      longitude,
    }
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
    self
      .inner
      .metadata
      .number_of_ensemble_members
      .map(|n| n as u32)
  }

  #[napi(getter)]
  pub fn data(&self) -> Vec<f64> {
    self.inner.data.clone()
  }

  /// Like the `data` getter: `adjustLongitudeRange` rolls columns to match a
  /// `[-180, 180)` longitude axis; `northUp` reverses rows so the 0th row is the
  /// northern-most. Pair with `latlngAdjusted` using the same flags. Each flag
  /// is a no-op when the grid is ineligible.
  #[napi]
  pub fn data_adjusted(&self, adjust_longitude_range: bool, north_up: Option<bool>) -> Vec<f64> {
    self.inner.metadata.projector.adjust_data(
      self.inner.data.clone(),
      adjust_longitude_range,
      north_up.unwrap_or(false),
    )
  }
}

/// One entry of a GRIB sidecar index file (NOAA wgrib2 `.idx` or ECMWF
/// open-data `.index`), locating a single message inside a GRIB file. Use the
/// offset/length as an HTTP Range request, then parse the fetched bytes with
/// `GribMessage.parseFromBuffer(bytes, 0)` — no full-file download needed.
#[napi(object)]
pub struct GribIndexEntry {
  /// 1-based message number (the line number for ECMWF indexes).
  pub message_number: u32,
  /// GRIB2 submessage number for NOAA `msg.submsg` lines; submessages share
  /// their parent message's byte range.
  pub submessage: Option<u32>,
  /// Byte offset of the message start within the GRIB file.
  pub offset: i64,
  /// Message size in bytes. Explicit in ECMWF indexes; inferred from the next
  /// entry's offset in NOAA indexes, so undefined for the final entry unless
  /// the GRIB file size is supplied.
  pub length: Option<i64>,
  /// Model reference (initialization) time.
  pub reference_date: Option<chrono::DateTime<chrono::Utc>>,
  /// Variable identifier: NOAA abbreviation ("TMP") or ECMWF MARS param ("2t").
  pub var: Option<String>,
  /// Level, verbatim ("2 m above ground", or ECMWF levelist).
  pub level: Option<String>,
  /// Forecast time, verbatim ("3 hour fcst", "anl", or ECMWF step).
  pub forecast_time: Option<String>,
  /// Trailing NOAA fields verbatim ("ENS=+5", probability info, ...).
  pub extra: Vec<String>,
  /// ECMWF MARS keys verbatim (levtype, stream, number, ...).
  pub keys: HashMap<String, String>,
}

/// Parse a GRIB sidecar index file (NOAA wgrib2 `.idx` or ECMWF open-data
/// `.index`, auto-detected) into entries locating each message. `fileSize`
/// (if known) sizes the final entry of a NOAA index.
#[napi]
pub fn parse_grib_index(data: String, file_size: Option<i64>) -> napi::Result<Vec<GribIndexEntry>> {
  let entries = parse_index(&data, file_size.map(|s| s as u64))
    .map_err(|e| napi::Error::from_reason(e.to_string()))?;
  Ok(
    entries
      .into_iter()
      .map(|entry| GribIndexEntry {
        message_number: entry.message_number as u32,
        submessage: entry.submessage.map(|s| s as u32),
        offset: entry.offset as i64,
        length: entry.length.map(|l| l as i64),
        reference_date: entry.reference_date,
        var: entry.var,
        level: entry.level,
        forecast_time: entry.forecast_time,
        extra: entry.extra,
        keys: entry.keys.into_iter().collect(),
      })
      .collect(),
  )
}

/// Wrap a `[0, 360)` longitude coordinate axis to a monotonic `[-180, 180)`
/// range, matching the column roll `dataAdjusted` applies to the values. A
/// no-op for axes that aren't eligible near-global ascending grids (returns the
/// input unchanged), so it is safe to call on any longitude array.
#[napi]
pub fn adjust_longitude_values(longitudes: Vec<f64>) -> Vec<f64> {
  adjust_longitude_values_core(longitudes)
}

/// Reverse a 1-D latitude coordinate axis so it descends from north to south,
/// matching the row reversal `dataAdjusted` applies under `northUp`. A no-op
/// for axes that already descend. Operates on a 1-D axis only; a flattened 2-D
/// latitude field would have its columns mirrored too.
#[napi]
pub fn adjust_latitude_values(latitudes: Vec<f64>) -> Vec<f64> {
  adjust_latitude_values_core(latitudes)
}

#[napi]
pub fn parse_messages_from_buffer(buffer: Uint8Array) -> Vec<GribMessage> {
  let buf = buffer.to_vec();
  read_messages(&buf)
    .filter_map(|gm| {
      DataMessage::try_from(&gm)
        .ok()
        .map(|msg| GribMessage { inner: msg })
    })
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
    let message =
      DataMessage::try_from(&message).map_err(|e| napi::Error::from_reason(e.to_string()))?;
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
    Ok(GribMessage {
      inner: data_message,
    })
  }
}
