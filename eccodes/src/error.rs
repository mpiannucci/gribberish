use thiserror::Error;

/// Errors that can occur when working with eccodes
#[derive(Error, Debug)]
pub enum EccodesError {
    #[error("Failed to create handle from buffer: {0}")]
    HandleCreationError(String),

    #[error("Failed to get key '{key}': {message}")]
    KeyAccessError { key: String, message: String },

    #[error("Failed to get message size")]
    MessageSizeError,

    #[error("Failed to decode values: {0}")]
    DecodeError(String),

    #[error("Invalid handle")]
    InvalidHandle,

    #[error("End of file reached")]
    EndOfFile,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Null pointer error: {0}")]
    NullError(#[from] std::ffi::NulError),

    #[error("Eccodes error code {code}: {message}")]
    EccodesNativeError { code: i32, message: String },
}

/// Result type for eccodes operations
pub type Result<T> = std::result::Result<T, EccodesError>;

/// Convert eccodes error code to descriptive message
pub fn error_code_to_string(code: i32) -> String {
    match code {
        0 => "Success".to_string(),
        -1 => "End of resource reached".to_string(),
        -2 => "Internal error".to_string(),
        -3 => "Buffer too small".to_string(),
        -4 => "Function not implemented".to_string(),
        -5 => "Missing 7777 at end of message".to_string(),
        -6 => "End of index reached".to_string(),
        -7 => "Internal array too small".to_string(),
        -8 => "Missing bufr entry".to_string(),
        -9 => "Null handle".to_string(),
        -10 => "Invalid key".to_string(),
        -11 => "Wrong array size".to_string(),
        -12 => "Key/value not found".to_string(),
        -13 => "Input output problem".to_string(),
        -14 => "Message invalid".to_string(),
        -15 => "Decoding invalid".to_string(),
        -16 => "Encoding invalid".to_string(),
        -17 => "Code not found in table".to_string(),
        -18 => "Array size mismatch".to_string(),
        -19 => "Key type error".to_string(),
        -20 => "Read only".to_string(),
        -21 => "Out of memory".to_string(),
        -22 => "Value cannot be missing".to_string(),
        -23 => "Wrong message length".to_string(),
        -24 => "Value is different".to_string(),
        -25 => "Invalid type".to_string(),
        -26 => "Unable to set step".to_string(),
        -27 => "Wrong step unit".to_string(),
        -28 => "Invalid file".to_string(),
        -29 => "Value mismatch".to_string(),
        -30 => "Double value is out of range".to_string(),
        -31 => "Underflow".to_string(),
        -32 => "Message malformed".to_string(),
        -33 => "Index corruption".to_string(),
        -34 => "Invalid key id".to_string(),
        -35 => "No more in set".to_string(),
        -36 => "Encoding error".to_string(),
        -37 => "No definitions".to_string(),
        -38 => "Wrong bitmap size".to_string(),
        _ => format!("Unknown error code: {}", code),
    }
}
