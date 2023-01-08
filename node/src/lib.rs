#![deny(clippy::all)]

use std::collections::HashMap;

use gribberish::{
  data_message::DataMessage,
  message::{scan_messages, read_message, read_messages},
};
use napi::{
  bindgen_prelude::{Array, Buffer, Float64Array},
  Env,
};

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct GridShape {
  pub rows: u32,
  pub cols: u32,
}

#[napi]
pub struct GribMessage {
  inner: DataMessage,
}

#[napi]
impl GribMessage {
  #[napi(factory)]
  pub fn parse_from_buffer(buffer: Buffer, offset: u32) -> Self {
    let buf: Vec<u8> = buffer.into();
    let message = read_message(&buf, offset as usize).unwrap();
    let message = DataMessage::try_from(&message).unwrap();

    GribMessage { inner: message }
  }

  pub fn parse_from_bytes(data: &[u8], offset: usize) -> Self {
    let message = read_message(&data, offset).unwrap();
    let message = DataMessage::try_from(&message).unwrap();

    GribMessage { inner: message }
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

  // #[napi(getter)]
  // pub fn array_index(&self) -> u32 {
  //   self.inner.metadata.array_index.unwrap_or(0) as u32
  // }

  #[napi(getter)]
  pub fn forecast_date(&self) -> chrono::DateTime<chrono::Utc> {
    self.inner.metadata.forecast_date
  }

  #[napi(getter)]
  pub fn reference_date(&self) -> chrono::DateTime<chrono::Utc> {
    self.inner.metadata.reference_date
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
  pub fn bbox(&self) -> Vec<f64> {
    let bbox = &self.inner.metadata.bbox;
    vec![bbox.0, bbox.1, bbox.2, bbox.3]
  }

  #[napi(getter)]
  pub fn grid_shape(&self) -> GridShape {
    let (rows, cols) = self.inner.metadata.grid_shape();
    GridShape {
      rows: rows as u32,
      cols: cols as u32,
    }
  }

  #[napi(getter)]
  pub fn grid_resolution(&self) -> GridShape {
    let (rows, cols) = self.inner.metadata.grid_resolution;
    GridShape {
      rows: rows as u32,
      cols: cols as u32,
    }
  }

  #[napi(getter)]
  pub fn latitudes(&self) -> Float64Array {
    Float64Array::new(self.inner.metadata.latitude.clone())
  }

  #[napi(getter)]
  pub fn longitudes(&self) -> Float64Array {
    Float64Array::new(self.inner.metadata.longitude.clone())
  }

  #[napi(getter)]
  pub fn data(&self) -> Float64Array {
    Float64Array::new(self.inner.flattened_data())
  }
}

#[napi]
pub fn parse_messages_from_buffer(buffer: Buffer, env: Env) -> Array {
  let buf: Vec<u8> = buffer.into();
  let messages = read_messages(&buf).collect::<Vec<_>>();

  let mut arr = env.create_array(0).unwrap();
  messages.into_iter().for_each(|gm| {
    let grib_message = GribMessage {
      inner: DataMessage::try_from(&gm).unwrap(),
    };

    arr.insert(grib_message).unwrap();
  });

  arr
}

#[napi]
pub struct GribMessageFactory {
  data: Vec<u8>,
  mapping: HashMap<String, (usize, usize)>,
}

#[napi]
impl GribMessageFactory {
  #[napi(factory)]
  pub fn from_buffer(buffer: Buffer) -> Self {
    let data: Vec<u8> = buffer.into();
    let mapping = scan_messages(&data);

    GribMessageFactory { data, mapping }
  }

  #[napi(getter)]
  pub fn available_messages(&self) -> Vec<String> {
    self.mapping.keys().map(|k| k.into()).collect()
  }

  #[napi]
  pub fn get_message(&self, key: String) -> GribMessage {
    GribMessage::parse_from_bytes(&self.data, self.mapping[&key].1)
  }
}
