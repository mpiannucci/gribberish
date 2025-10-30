use crate::error::{error_code_to_string, EccodesError, Result};
use eccodes_sys::*;
use std::ffi::CString;
use std::ptr;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize eccodes with multi-field support enabled
fn init_eccodes() {
    INIT.call_once(|| {
        unsafe {
            // Enable multi-field support in GRIB messages
            // NULL means use the default context
            codes_grib_multi_support_on(ptr::null_mut());
        }
    });
}

/// Safe wrapper around eccodes grib_handle
///
/// Automatically manages the lifecycle of the eccodes handle,
/// ensuring proper cleanup when dropped.
pub struct GribHandle {
    handle: *mut codes_handle,
}

impl GribHandle {
    /// Create a new GribHandle from a byte buffer
    ///
    /// # Arguments
    /// * `data` - The GRIB message data as a byte slice
    ///
    /// # Returns
    /// A new GribHandle or an error if the handle could not be created
    pub fn new_from_message(data: &[u8]) -> Result<Self> {
        // Ensure eccodes is initialized with multi-field support
        init_eccodes();

        unsafe {
            // codes_handle_new_from_message returns NULL on error
            let handle = codes_handle_new_from_message(
                ptr::null_mut(),
                data.as_ptr() as *const std::os::raw::c_void,
                data.len(),
            );

            if handle.is_null() {
                return Err(EccodesError::HandleCreationError(
                    "Failed to create handle from message".to_string(),
                ));
            }

            Ok(GribHandle { handle })
        }
    }

    /// Get a string value from the GRIB message
    ///
    /// # Arguments
    /// * `key` - The eccodes key name (e.g., "shortName", "paramId")
    ///
    /// # Returns
    /// The string value or an error
    pub fn get_string(&self, key: &str) -> Result<String> {
        if self.handle.is_null() {
            return Err(EccodesError::InvalidHandle);
        }

        let key_cstr = CString::new(key)?;
        let mut size: usize = 0;

        unsafe {
            // Get the size needed for the string
            let mut err = codes_get_length(self.handle, key_cstr.as_ptr(), &mut size);
            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::KeyAccessError {
                    key: key.to_string(),
                    message: error_code_to_string(err),
                });
            }

            // Allocate buffer and get the string
            let mut buffer = vec![0u8; size];
            err = codes_get_string(
                self.handle,
                key_cstr.as_ptr(),
                buffer.as_mut_ptr() as *mut i8,
                &mut size,
            );

            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::KeyAccessError {
                    key: key.to_string(),
                    message: error_code_to_string(err),
                });
            }

            // Convert to Rust string, removing null terminator
            buffer.truncate(size.saturating_sub(1));
            Ok(String::from_utf8(buffer)?)
        }
    }

    /// Get a long (i64) value from the GRIB message
    ///
    /// # Arguments
    /// * `key` - The eccodes key name
    ///
    /// # Returns
    /// The long value or an error
    pub fn get_long(&self, key: &str) -> Result<i64> {
        if self.handle.is_null() {
            return Err(EccodesError::InvalidHandle);
        }

        let key_cstr = CString::new(key)?;
        let mut value: i64 = 0;

        unsafe {
            let err = codes_get_long(self.handle, key_cstr.as_ptr(), &mut value);
            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::KeyAccessError {
                    key: key.to_string(),
                    message: error_code_to_string(err),
                });
            }
        }

        Ok(value)
    }

    /// Get a double (f64) value from the GRIB message
    ///
    /// # Arguments
    /// * `key` - The eccodes key name
    ///
    /// # Returns
    /// The double value or an error
    pub fn get_double(&self, key: &str) -> Result<f64> {
        if self.handle.is_null() {
            return Err(EccodesError::InvalidHandle);
        }

        let key_cstr = CString::new(key)?;
        let mut value: f64 = 0.0;

        unsafe {
            let err = codes_get_double(self.handle, key_cstr.as_ptr(), &mut value);
            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::KeyAccessError {
                    key: key.to_string(),
                    message: error_code_to_string(err),
                });
            }
        }

        Ok(value)
    }

    /// Get an array of double values from the GRIB message
    ///
    /// # Arguments
    /// * `key` - The eccodes key name (typically "values" for data array)
    ///
    /// # Returns
    /// A vector of double values or an error
    pub fn get_double_array(&self, key: &str) -> Result<Vec<f64>> {
        if self.handle.is_null() {
            return Err(EccodesError::InvalidHandle);
        }

        let key_cstr = CString::new(key)?;
        let mut size: usize = 0;

        unsafe {
            // Get the size of the array
            let mut err = codes_get_size(self.handle, key_cstr.as_ptr(), &mut size);
            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::KeyAccessError {
                    key: key.to_string(),
                    message: error_code_to_string(err),
                });
            }

            // Allocate buffer and get the values
            let mut values = vec![0.0f64; size];
            err = codes_get_double_array(
                self.handle,
                key_cstr.as_ptr(),
                values.as_mut_ptr(),
                &mut size,
            );

            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::DecodeError(error_code_to_string(err)));
            }

            Ok(values)
        }
    }

    /// Get the decoded data values from the GRIB message
    ///
    /// This is a convenience method that calls get_double_array("values")
    pub fn get_data(&self) -> Result<Vec<f64>> {
        self.get_double_array("values")
    }

    /// Get the size of the message in bytes
    pub fn get_message_size(&self) -> Result<usize> {
        if self.handle.is_null() {
            return Err(EccodesError::InvalidHandle);
        }

        unsafe {
            let mut size: usize = 0;
            let err = codes_get_message_size(self.handle, &mut size);
            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::MessageSizeError);
            }
            Ok(size)
        }
    }

    /// Clone the message data into a new buffer
    pub fn get_message_bytes(&self) -> Result<Vec<u8>> {
        let size = self.get_message_size()?;
        let mut buffer = vec![0u8; size];

        unsafe {
            let mut msg_size = size;
            let err = codes_get_message(
                self.handle,
                buffer.as_mut_ptr() as *mut *const std::os::raw::c_void,
                &mut msg_size,
            );
            if err != CODES_SUCCESS as i32 {
                return Err(EccodesError::DecodeError(error_code_to_string(err)));
            }
        }

        Ok(buffer)
    }

    /// Get the raw handle pointer (for advanced use cases)
    ///
    /// # Safety
    /// The caller must ensure the handle is not used after this GribHandle is dropped
    pub unsafe fn as_ptr(&self) -> *mut codes_handle {
        self.handle
    }
}

impl Drop for GribHandle {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                codes_handle_delete(self.handle);
            }
            self.handle = ptr::null_mut();
        }
    }
}

// GribHandle is Send because eccodes handles can be safely sent between threads
// (each thread gets its own handle)
unsafe impl Send for GribHandle {}

// GribHandle is not Sync because eccodes handles should not be shared between threads
// without synchronization
