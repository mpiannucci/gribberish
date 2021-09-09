mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use js_sys::{Array, Date, Float64Array};
use gribberish::message::Message;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct LatLon {
    pub lat: f64, 
    pub lon: f64,
}

#[wasm_bindgen]
pub struct Region {
    #[wasm_bindgen(js_name = topLeft)]
    pub top_left: LatLon, 
    #[wasm_bindgen(js_name = bottomRight)]
    pub bottom_right: LatLon,
}

#[wasm_bindgen]
pub struct GridShape {
    pub rows: usize, 
    pub cols: usize,
}

#[wasm_bindgen]
pub struct GribMessage {
    inner: Message,
}

#[wasm_bindgen]
impl GribMessage {
    #[wasm_bindgen(method, getter = varName)]
    pub fn var_name(&self) -> String {
        self.inner.variable_name().unwrap()
    }
    
    #[wasm_bindgen(method, getter = varAbbrev)]
    pub fn var_abbrev(&self) -> String {
        self.inner.variable_abbrev().unwrap()
    }

    #[wasm_bindgen(method, getter)]
    pub fn units(&self) -> String {
        self.inner.unit().unwrap()
    }

    #[wasm_bindgen(method, getter = arrayIndex)]
    pub fn array_index(&self) -> Option<usize> {
        match self.inner.array_index() {
            Ok(i) => i, 
            Err(_) => None,
        }
    }

    #[wasm_bindgen(method, getter)]
    pub fn region(&self) -> Region {
        let region = self.inner.location_region().unwrap();
        Region {
            top_left: LatLon {
                lat: region.0.0, 
                lon: region.0.1,
            }, 
            bottom_right: LatLon {
                lat: region.1.0, 
                lon: region.1.1,
            },
        }
    }

    #[wasm_bindgen(method, getter = gridShape)]
    pub fn grid_shape(&self) -> GridShape {
        let shape = self.inner.location_grid_dimensions().unwrap();
        GridShape {
            rows: shape.0, 
            cols: shape.1,
        }
    }

    #[wasm_bindgen(method, getter = forecastDate)]
    pub fn forecast_date(&self) -> Date {
        let date = self.inner.forecast_date().unwrap();
        let timestamp = JsValue::from_f64(date.timestamp_millis() as f64);
        Date::new(&timestamp)
    }

    #[wasm_bindgen(method, getter = referenceDate)]
    pub fn reference_date(&self) -> Date {
        let date = self.inner.reference_date().unwrap();
        let timestamp = JsValue::from_f64(date.timestamp_millis() as f64);
        Date::new(&timestamp)
    }

    #[wasm_bindgen(js_name = dataAtLocation)]
    pub fn data_at_location(&self, lat: f64, lon: f64) -> Option<f64> {
        match self.inner.data_at_location(&(lat, lon)) {
            Ok(d) => Some(d), 
            Err(_) => None,
        }
    }

    pub fn data(&self) -> Float64Array {
        let raw_data = self.inner.data().unwrap();
        Float64Array::from(&raw_data[..])
    }

    #[wasm_bindgen(js_name = locationDataIndex)]
    pub fn location_data_index(&self, lat: f64, lon: f64) -> Option<usize> {
        match self.inner.data_index_for_location(&(lat, lon)) {
            Ok(i) => Some(i), 
            Err(_) => None,
        }
    }
}

#[wasm_bindgen(js_name = parseGribMessage)]
pub fn parse_grib_message(data: &[u8], offset: usize) -> GribMessage {
    let message = Message::parse(data, offset).unwrap();
    GribMessage {
        inner: message,
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Array<GribMessage>")]
    pub type GribMessageArray;
}

#[wasm_bindgen(js_name = parseGribMessages)]
pub fn parse_grib_messages(data: &[u8]) -> GribMessageArray {
    let mut messages = Message::parse_all(data);
    let mut contained_messages = Vec::new();
    while messages.len() > 0 {
        let grib_message = GribMessage {
            inner: messages.pop().unwrap()
        };

        contained_messages.push(grib_message);
    }

    contained_messages
        .into_iter()
        .map(JsValue::from)
        .collect::<Array>()
        .unchecked_into::<GribMessageArray>()
}
