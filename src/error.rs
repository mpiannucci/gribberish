use thiserror::Error;

#[derive(Clone, Debug, Error)]
pub enum GribberishError {
    #[error("Error reading data representation template metadata: `{0}`")]
    DataRepresentationTemplateError(String),
    #[error("Error reading grid template metadata: `{0}`")]
    GridTemplateError(String),
    #[error("Error decoding JPEG2000 data: `{0}`")]
    JpegError(String),
    #[error("Error reading message: `{0}`")]
    MessageError(String),
    #[error("Unknown time unit: `{0}`")]
    TimeUnitError(String),
}
