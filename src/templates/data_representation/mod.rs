pub mod tables;
pub mod data_representation_template;
pub mod simple_packing_template;
pub mod complex_packing_template;
pub mod complex_spatial_packing_template;

#[cfg(feature = "jpeg")]
pub mod jpeg_template;

#[cfg(feature = "png")]
pub mod png_template;

pub use data_representation_template::DataRepresentationTemplate;
pub use simple_packing_template::SimplePackingDataRepresentationTemplate;
pub use complex_packing_template::ComplexPackingDataRepresentationTemplate;
pub use complex_spatial_packing_template::ComplexSpatialPackingDataRepresentationTemplate;

#[cfg(feature = "jpeg")]
pub use jpeg_template::JPEGDataRepresentationTemplate;

#[cfg(feature = "png")]
pub use png_template::PNGDataRepresentationTemplate;