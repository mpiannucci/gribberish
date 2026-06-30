use std::convert::TryInto;
use std::ffi::c_void;
use std::ptr::null_mut;
use std::slice;

use crate::error::GribberishError;

// https://github.com/ecmwf/eccodes/blob/develop/src/grib_openjpeg_encoding.c
// https://github.com/leoschwarz/jpeg2000-rust/blob/master/src/decode/mod.rs

pub struct JpegUserData<'a> {
    input_stream: bool,
    offset: usize,
    output: Vec<u8>,
    input: &'a [u8],
}

impl<'a> JpegUserData<'a> {
    pub fn new_input(data: &'a [u8]) -> Self {
        JpegUserData {
            input_stream: true,
            offset: 0,
            output: Vec::new(),
            input: data,
        }
    }
}

pub unsafe extern "C" fn jpeg_opj_stream_read_fn(
    p_buffer: *mut c_void,
    p_nb_bytes: usize,
    p_user_data: *mut c_void,
) -> usize {
    let userdata = &mut *(p_user_data as *mut JpegUserData);
    assert!(userdata.input_stream);

    let n_imgsize = userdata.input.len();
    let n_byteleft = n_imgsize - userdata.offset;

    let mut n_read = p_nb_bytes;

    if n_read > n_byteleft {
        n_read = n_byteleft;
    }

    if userdata.input.is_empty() || p_buffer.is_null() || n_read == 0 || n_byteleft == 0 {
        // TODO: The original returned -1 here,
        // but for some reason our signature is usize...
        return 0;
    }

    let target = slice::from_raw_parts_mut(p_buffer as *mut u8, n_read);
    let offset = userdata.offset;
    target.copy_from_slice(&userdata.input[offset..offset + n_read]);

    userdata.offset += n_read;

    n_read
}

pub unsafe extern "C" fn jpeg_opj_stream_write_fn(
    p_buffer: *mut c_void,
    p_nb_bytes: usize,
    p_user_data: *mut c_void,
) -> usize {
    let userdata = p_user_data as *mut JpegUserData;
    assert!(!(*userdata).input_stream);

    let buffer = p_buffer as *mut u8;

    (*userdata)
        .output
        .reserve((*userdata).output.len() + p_nb_bytes);
    (*userdata)
        .output
        .extend_from_slice(slice::from_raw_parts(buffer, p_nb_bytes));

    p_nb_bytes
}

pub unsafe extern "C" fn jpeg_opj_stream_skip_fn(p_nb_bytes: i64, p_user_data: *mut c_void) -> i64 {
    let userdata = &mut *(p_user_data as *mut JpegUserData);
    assert!(userdata.input_stream);

    let n_imgsize = userdata.input.len();
    let n_byteleft = (n_imgsize - userdata.offset) as i64;

    let mut n_skip = p_nb_bytes;

    if n_skip > n_byteleft {
        n_skip = n_byteleft;
    }

    userdata.offset += n_skip as usize;
    userdata.offset as i64
}

pub unsafe extern "C" fn jpeg_opj_stream_seek_fn(p_nb_bytes: i64, p_user_data: *mut c_void) -> i32 {
    let userdata = &mut *(p_user_data as *mut JpegUserData);
    assert!(userdata.input_stream);

    let n_imgsize = userdata.input.len();
    let n_seek = p_nb_bytes as usize;

    if n_seek > n_imgsize {
        0
    } else {
        userdata.offset = n_seek;
        1
    }
}

// RAII guards so each openjpeg resource is freed on every exit path, including
// the early returns below. This makes a missing destroy (the bug these wrap)
// impossible to reintroduce. All three opj_*_destroy functions are null-safe,
// so dropping a guard around a null pointer (e.g. the image before the header
// is read, or a failed create) is fine.
struct OpjCodec(*mut openjpeg_sys::opj_codec_t);
impl Drop for OpjCodec {
    fn drop(&mut self) {
        unsafe { openjpeg_sys::opj_destroy_codec(self.0) }
    }
}

struct OpjStream(*mut openjpeg_sys::opj_stream_t);
impl Drop for OpjStream {
    fn drop(&mut self) {
        unsafe { openjpeg_sys::opj_stream_destroy(self.0) }
    }
}

struct OpjImage(*mut openjpeg_sys::opj_image);
impl Drop for OpjImage {
    fn drop(&mut self) {
        unsafe { openjpeg_sys::opj_image_destroy(self.0) }
    }
}

pub fn extract_jpeg_data(raw_data: &[u8]) -> Result<Vec<i32>, GribberishError> {
    let output_data: Vec<i32>;

    unsafe {
        let parameters = &mut openjpeg_sys::opj_dparameters {
            cp_reduce: 0,
            cp_layer: 0,
            infile: [0; 4096],
            outfile: [0; 4096],
            decod_format: 0,
            cod_format: 0,
            DA_x0: 0,
            DA_x1: 0,
            DA_y0: 0,
            DA_y1: 0,
            m_verbose: 0,
            tile_index: 0,
            nb_tile_to_decode: 0,
            jpwl_correct: 0,
            jpwl_exp_comps: 0,
            jpwl_max_tiles: 0,
            flags: 0,
        } as *mut openjpeg_sys::opj_dparameters_t;
        openjpeg_sys::opj_set_default_decoder_parameters(parameters);
        let codec = OpjCodec(openjpeg_sys::opj_create_decompress(
            openjpeg_sys::CODEC_FORMAT::OPJ_CODEC_J2K,
        ));
        openjpeg_sys::opj_setup_decoder(codec.0, parameters);

        let mut userdata = JpegUserData::new_input(raw_data);
        let stream = OpjStream(openjpeg_sys::opj_stream_default_create(1));
        openjpeg_sys::opj_stream_set_read_function(stream.0, Some(jpeg_opj_stream_read_fn));
        openjpeg_sys::opj_stream_set_write_function(stream.0, Some(jpeg_opj_stream_write_fn));
        openjpeg_sys::opj_stream_set_skip_function(stream.0, Some(jpeg_opj_stream_skip_fn));
        openjpeg_sys::opj_stream_set_seek_function(stream.0, Some(jpeg_opj_stream_seek_fn));

        let userdata_ptr: *mut JpegUserData = &mut userdata;
        openjpeg_sys::opj_stream_set_user_data_length(stream.0, raw_data.len().try_into().unwrap());
        openjpeg_sys::opj_stream_set_user_data(stream.0, userdata_ptr as *mut c_void, None);

        let mut image = OpjImage(null_mut());
        if openjpeg_sys::opj_read_header(stream.0, codec.0, &mut image.0) != 1 {
            return Err(GribberishError::JpegError(
                "Failed to decode JPEG byte stream header".into(),
            ));
        }

        if openjpeg_sys::opj_decode(codec.0, stream.0, image.0) != 1 {
            return Err(GribberishError::JpegError(
                "Failed to decode JPEG byte stream".into(),
            ));
        }

        // Do things to the data
        let comp = (*image.0).comps.offset(0);
        let component_data = (*comp).data;
        let mask = (1 << (*comp).prec) - 1;

        let count = (*comp).w * (*comp).h;

        let mut data = Vec::new();
        for i in 0..count {
            let index: isize = i.try_into().unwrap();
            let data_point = *component_data.offset(index) & mask;
            data.push(data_point);
        }
        output_data = data;

        // codec, stream, and image are dropped here, freeing the openjpeg
        // resources on this path and on every early return above.
    }

    if output_data.is_empty() {
        Err(GribberishError::JpegError(
            "Unknown failure extracting JPEG data".into(),
        ))
    } else {
        Ok(output_data)
    }
}
