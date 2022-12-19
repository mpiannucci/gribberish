pub mod tables;
pub mod data_representation_template;
pub mod simple_grid_point_template;

#[cfg(feature = "jpeg")]
pub mod jpeg_template;

#[cfg(feature = "png")]
pub mod png_template;

pub use data_representation_template::DataRepresentationTemplate;
pub use simple_grid_point_template::SimpleGridPointDataRepresentationTemplate;

#[cfg(feature = "jpeg")]
pub use jpeg_template::JPEGDataRepresentationTemplate;

#[cfg(feature = "png")]
pub use png_template::PNGDataRepresentationTemplate;