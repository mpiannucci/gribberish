use crate::error::Result;
use crate::handle::GribHandle;

/// Iterator over GRIB messages in a buffer using eccodes
///
/// This provides a safe, zero-copy way to iterate through multiple
/// GRIB messages in a byte buffer.
pub struct GribMessageIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> GribMessageIterator<'a> {
    /// Create a new iterator from a byte buffer
    ///
    /// # Arguments
    /// * `data` - The buffer containing one or more GRIB messages
    pub fn new(data: &'a [u8]) -> Self {
        GribMessageIterator { data, offset: 0 }
    }

    /// Get the current offset in the buffer
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Skip to a specific offset in the buffer
    pub fn seek(&mut self, offset: usize) {
        self.offset = offset;
    }
}

impl<'a> Iterator for GribMessageIterator<'a> {
    type Item = Result<(GribHandle, usize)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset >= self.data.len() {
            return None;
        }

        // Try to find the next GRIB message
        let remaining = &self.data[self.offset..];

        // Look for GRIB indicator (starts with "GRIB")
        let grib_start = remaining
            .windows(4)
            .position(|w| w == b"GRIB");

        let message_start = match grib_start {
            Some(pos) => self.offset + pos,
            None => return None,
        };

        // Skip to the GRIB message
        self.offset = message_start;
        let message_data = &self.data[self.offset..];

        // Try to create a handle from this position
        match GribHandle::new_from_message(message_data) {
            Ok(handle) => {
                let current_offset = self.offset;

                // Get the message size and advance the offset
                match handle.get_message_size() {
                    Ok(size) => {
                        self.offset += size;
                        Some(Ok((handle, current_offset)))
                    }
                    Err(e) => {
                        // If we can't get the size, try to advance past "GRIB" and continue
                        self.offset += 4;
                        Some(Err(e))
                    }
                }
            }
            Err(e) => {
                // If handle creation failed, try to skip past this position
                self.offset += 4;
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iterator_empty_buffer() {
        let data = b"";
        let mut iter = GribMessageIterator::new(data);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_iterator_no_grib() {
        let data = b"This is not a GRIB file";
        let mut iter = GribMessageIterator::new(data);
        assert!(iter.next().is_none());
    }
}
