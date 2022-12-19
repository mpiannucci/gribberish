#![deny(clippy::all)]

use gribberish::{
  data_message::DataMessage,
  message::{self, read_message, read_messages, Message},
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
    let message = DataMessage::try_from(message).unwrap();

    GribMessage { inner: message }
  }

  #[napi(getter)]
  pub fn var_name(&self) -> &str {
    self.inner.name.as_str()
  }

  #[napi(getter)]
  pub fn var_abbrev(&self) -> &str {
    self.inner.var.as_str()
  }

  #[napi(getter)]
  pub fn units(&self) -> &str {
    self.inner.units.as_str()
  }

  #[napi(getter)]
  pub fn array_index(&self) -> u32 {
    self.inner.array_index.unwrap_or(0) as u32
  }

  #[napi(getter)]
  pub fn forecast_date(&self) -> chrono::DateTime<chrono::Utc> {
    self.inner.forecast_date
  }

  #[napi(getter)]
  pub fn reference_date(&self) -> chrono::DateTime<chrono::Utc> {
    self.inner.reference_date
  }

  #[napi(getter)]
  pub fn proj(&self) -> &str {
    self.inner.proj.as_str()
  }

  #[napi(getter)]
  pub fn crs(&self) -> &str {
    self.inner.crs.as_str()
  }

  #[napi(getter)]
  pub fn bbox(&self) -> Vec<f64> {
    let bbox = &self.inner.bbox;
    vec![bbox.0, bbox.1, bbox.2, bbox.3]
  }

  #[napi(getter)]
  pub fn grid_shape(&self) -> GridShape {
    let (rows, cols) = self.inner.grid_shape();
    GridShape {
      rows: rows as u32, 
      cols: cols as u32
    }
  }

  #[napi(getter)]
  pub fn grid_resolution(&self) -> GridShape {
    let (rows, cols) = self.inner.grid_resolution;
    GridShape {
      rows: rows as u32, 
      cols: cols as u32
    }
  }

  #[napi(getter)]
  pub fn latitudes(&self) -> Float64Array {
    Float64Array::new(self.inner.latitude.clone())
  }

  #[napi(getter)]
  pub fn longitudes(&self) -> Float64Array {
    Float64Array::new(self.inner.longitude.clone())
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
      inner: DataMessage::try_from(gm).unwrap(),
    };

    arr.insert(grib_message).unwrap();
  });

  arr
}
