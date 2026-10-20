//! A [`zarrs`] array-to-bytes codec that decodes GRIB messages into Zarr chunks.
//!
//! This is the Rust counterpart of the Python `gribberish.zarr.GribberishCodec`
//! and reads the same `"gribberish"` codec metadata, so virtual Zarr stores
//! written by the Python VirtualiZarr parser can be read with `zarrs`. Each
//! chunk holds the raw bytes of one GRIB message.
//!
//! The codec registers itself with `zarrs` at link time, so enabling the
//! `zarrs` feature is enough for `zarrs` to open arrays that use it. It is
//! read-only: encoding is not supported.
//!
//! Configuration (all fields optional):
//!
//! - `var`: `"latitude"` or `"longitude"` decodes that coordinate from the
//!   message's grid instead of the data values.
//! - `adjust_longitude_range`: rewrap global 0–360° longitude grids to a
//!   monotonic −180…180° range. A no-op for grids that don't span the globe.
//! - `north_up`: reorder rows so the 0th row is the northern-most. A no-op for
//!   grids that are already north-first.

use std::borrow::Cow;
use std::num::NonZeroU64;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use zarrs::array::codec::api::{
    ArrayBytes, ArrayBytesRaw, ArrayCodecTraits, ArrayToBytesCodecTraits, BytesRepresentation,
    Codec, CodecError, CodecMetadataOptions, CodecOptions, CodecPluginV3, CodecTraits,
    CodecTraitsV3, PartialDecoderCapability, PartialEncoderCapability, RecommendedConcurrency,
};
use zarrs::array::data_type::{Float32DataType, Float64DataType};
use zarrs::array::{DataType, FillValue};
use zarrs::metadata::v3::MetadataV3;
use zarrs::metadata::{Configuration, ConfigurationSerialize};
use zarrs::plugin::{PluginCreateError, ZarrVersion};

use crate::message::Message;

zarrs::plugin::impl_extension_aliases!(GribberishCodec, v3: "gribberish");

inventory::submit! {
    CodecPluginV3::new::<GribberishCodec>()
}

/// Configuration for [`GribberishCodec`], matching the Python codec's metadata.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GribberishCodecConfiguration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub var: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub adjust_longitude_range: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub north_up: bool,
}

impl ConfigurationSerialize for GribberishCodecConfiguration {}

/// Which values a chunk decodes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GribVariable {
    Data,
    Latitude,
    Longitude,
}

/// A read-only `zarrs` codec that decodes a chunk of GRIB message bytes.
#[derive(Debug, Clone)]
pub struct GribberishCodec {
    configuration: GribberishCodecConfiguration,
    variable: GribVariable,
}

impl GribberishCodec {
    /// Create a new codec from its configuration.
    ///
    /// Any `var` other than `"latitude"` or `"longitude"` decodes the
    /// message's data values, as the Python codec does.
    pub fn new_with_configuration(configuration: &GribberishCodecConfiguration) -> Self {
        let variable = match configuration.var.as_deref() {
            Some("latitude") => GribVariable::Latitude,
            Some("longitude") => GribVariable::Longitude,
            _ => GribVariable::Data,
        };
        Self {
            configuration: configuration.clone(),
            variable,
        }
    }

    /// Decode the message in `bytes` to `f64` values in grid order.
    fn decode_values(&self, bytes: &[u8]) -> Result<Vec<f64>, CodecError> {
        let message = Message::from_data(bytes, 0)
            .ok_or_else(|| CodecError::Other("Failed to read GRIB message".to_string()))?;
        let GribberishCodecConfiguration {
            adjust_longitude_range,
            north_up,
            ..
        } = self.configuration;

        let projector = || {
            message
                .latlng_projector()
                .map_err(|e| CodecError::Other(format!("Failed to build projection: {e}")))
        };

        match self.variable {
            GribVariable::Latitude | GribVariable::Longitude => {
                let (lat, lng) = projector()?.lat_lng_adjusted(adjust_longitude_range, north_up);
                Ok(if self.variable == GribVariable::Latitude {
                    lat
                } else {
                    lng
                })
            }
            GribVariable::Data => {
                let values = message
                    .data()
                    .map_err(|e| CodecError::Other(format!("Failed to decode GRIB data: {e}")))?;
                // The projector decides eligibility for both adjustments so data
                // and coordinates stay aligned; build it only when needed.
                if adjust_longitude_range || north_up {
                    Ok(projector()?.adjust_data(values, adjust_longitude_range, north_up))
                } else {
                    Ok(values)
                }
            }
        }
    }
}

impl CodecTraitsV3 for GribberishCodec {
    fn create(metadata: &MetadataV3) -> Result<Codec, PluginCreateError> {
        let configuration: GribberishCodecConfiguration = metadata.to_typed_configuration()?;
        let codec = Arc::new(GribberishCodec::new_with_configuration(&configuration));
        Ok(Codec::ArrayToBytes(codec))
    }
}

impl CodecTraits for GribberishCodec {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn configuration(
        &self,
        _version: ZarrVersion,
        _options: &CodecMetadataOptions,
    ) -> Option<Configuration> {
        Some(self.configuration.clone().into())
    }

    fn partial_decoder_capability(&self) -> PartialDecoderCapability {
        PartialDecoderCapability {
            partial_read: false,
            partial_decode: false,
        }
    }

    fn partial_encoder_capability(&self) -> PartialEncoderCapability {
        PartialEncoderCapability {
            partial_encode: false,
        }
    }
}

impl ArrayCodecTraits for GribberishCodec {
    fn recommended_concurrency(
        &self,
        _shape: &[NonZeroU64],
        _data_type: &DataType,
    ) -> Result<RecommendedConcurrency, CodecError> {
        Ok(RecommendedConcurrency::new_maximum(1))
    }
}

impl ArrayToBytesCodecTraits for GribberishCodec {
    fn into_dyn(self: Arc<Self>) -> Arc<dyn ArrayToBytesCodecTraits> {
        self as Arc<dyn ArrayToBytesCodecTraits>
    }

    fn encoded_representation(
        &self,
        _shape: &[NonZeroU64],
        _data_type: &DataType,
        _fill_value: &FillValue,
    ) -> Result<BytesRepresentation, CodecError> {
        Ok(BytesRepresentation::UnboundedSize)
    }

    fn encode<'a>(
        &self,
        _bytes: ArrayBytes<'a>,
        _shape: &[NonZeroU64],
        _data_type: &DataType,
        _fill_value: &FillValue,
        _options: &CodecOptions,
    ) -> Result<ArrayBytesRaw<'a>, CodecError> {
        Err(CodecError::Other(
            "the gribberish codec is read-only and does not support encoding".to_string(),
        ))
    }

    fn decode<'a>(
        &self,
        bytes: ArrayBytesRaw<'a>,
        shape: &[NonZeroU64],
        data_type: &DataType,
        _fill_value: &FillValue,
        _options: &CodecOptions,
    ) -> Result<ArrayBytes<'a>, CodecError> {
        let values = self.decode_values(&bytes)?;

        let num_elements: u64 = shape.iter().map(|d| d.get()).product();
        if values.len() as u64 != num_elements {
            return Err(CodecError::Other(format!(
                "GRIB message decoded to {} values, but the chunk shape {:?} holds {num_elements}",
                values.len(),
                shape
            )));
        }

        // Decoded array bytes are native-endian.
        let bytes: Vec<u8> = if data_type.is::<Float64DataType>() {
            values.iter().flat_map(|v| v.to_ne_bytes()).collect()
        } else if data_type.is::<Float32DataType>() {
            values
                .iter()
                .flat_map(|v| (*v as f32).to_ne_bytes())
                .collect()
        } else {
            return Err(CodecError::Other(format!(
                "the gribberish codec only decodes to float32 or float64, not {data_type}"
            )));
        };

        Ok(ArrayBytes::from(Cow::Owned(bytes)))
    }
}
