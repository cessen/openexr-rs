extern crate libc;

use libc::{c_char, c_int, c_double, size_t, c_void};

#[repr(C)]
pub enum CEXR_PixelType {
    UINT = 0, // unsigned int (32 bit)
    HALF = 1, // half (16 bit floating point)
    FLOAT = 2, // float (32 bit floating point)
}

#[repr(C)]
pub enum CEXR_CompressionMethod {
    NO_COMPRESSION = 0, // no compression
    RLE_COMPRESSION = 1, // run length encoding
    ZIPS_COMPRESSION = 2, // zlib compression, one scan line at a time
    ZIP_COMPRESSION = 3, // zlib compression, in blocks of 16 scan lines
    PIZ_COMPRESSION = 4, // piz-based wavelet compression
    PXR24_COMPRESSION = 5, // lossy 24-bit float compression
    B44_COMPRESSION = 6, // lossy 4-by-4 pixel block compression,
    // fixed compression rate
    B44A_COMPRESSION = 7, // lossy 4-by-4 pixel block compression,
    // flat fields are compressed more
    DWAA_COMPRESSION = 8, // lossy DCT based compression, in blocks
    // of 32 scanlines. More efficient for partial
    // buffer access.
    DWAB_COMPRESSION = 9, /* lossy DCT based compression, in blocks
                           * of 256 scanlines. More efficient space
                           * wise and faster to decode full frames
                           * than DWAA_COMPRESSION. */
}

#[repr(C)]
pub enum CEXR_LineOrder {
    INCREASING_Y = 0, // first scan line has lowest y coordinate
    DECREASING_Y = 1, // first scan line has highest y coordinate
    RANDOM_Y = 2, /* only for tiled files; tiles are written
                   * in random order */
}

#[repr(C)]
pub struct CEXR_Channel {
    pub pixel_type: CEXR_PixelType,
    pub x_sampling: c_int,
    pub y_sampling: c_int,
    pub p_linear: c_int, // bool
}


// ------------------------------------------------------------------------------
// Channel iterator
#[repr(C)]
pub struct CEXR_ChannelIterator {
    begin: *mut c_void,
    end: *mut c_void,
}

extern "C" {
    pub fn CEXR_ChannelIterator_delete(iterator: *mut CEXR_ChannelIterator);
    pub fn CEXR_ChannelIterator_next(iterator: *mut CEXR_ChannelIterator) -> *const c_char;
}


// ------------------------------------------------------------------------------
// EXR header type.
#[repr(C)]
pub struct CEXR_Header {
    header: *mut c_void,
}

extern "C" {
    pub fn CEXR_Header_delete(header: *mut CEXR_Header);
    pub fn CEXR_Header_insert_channel(header: *mut CEXR_Header,
                                      name: *const c_char,
                                      channel: CEXR_Channel);
    pub fn CEXR_Header_channel_exists(header: *const CEXR_Header, name: *const c_char) -> c_int;
    pub fn CEXR_Header_get_channel(header: *const CEXR_Header,
                                   name: *const c_char)
                                   -> CEXR_Channel;
    pub fn CEXR_Header_new_channel_iterator(header: *const CEXR_Header) -> CEXR_ChannelIterator;
}


// ------------------------------------------------------------------------------
// FrameBuffer
#[repr(C)]
pub struct CEXR_FrameBuffer {
    frame_buffer: *mut c_void,
}

extern "C" {
    pub fn CEXR_FrameBuffer_new() -> CEXR_FrameBuffer;
    pub fn CEXR_FrameBuffer_delete(frame_buffer: *mut CEXR_FrameBuffer);
    pub fn CEXR_FrameBuffer_insert_slice(frame_buffer: *mut CEXR_FrameBuffer,
                                         name: *const c_char,
                                         pixel_type: CEXR_PixelType,
                                         base: *mut c_char,
                                         x_stride: size_t,
                                         y_stride: size_t,
                                         x_sampling: c_int,
                                         y_sampling: c_int,
                                         fill_value: c_double,
                                         x_tile_coords: c_int, // bool
                                         y_tile_coords: c_int /* bool */);
}


// ------------------------------------------------------------------------------
// OutputFile
#[repr(C)]
pub struct CEXR_OutputFile {
    output_file: *mut c_void,
}

extern "C" {
    pub fn CEXR_OutputFile_new(file_name: *const c_char, header: *const CEXR_Header, num_threads: c_int) -> CEXR_OutputFile;
    pub fn CEXR_OutputFile_delete(output_file: *mut CEXR_OutputFile);
    pub fn CEXR_OutputFile_set_frame_buffer(output_file: *mut CEXR_OutputFile, frame_buffer: *mut CEXR_FrameBuffer);
    pub fn CEXR_OutputFile_write_pixels(output_file: *mut CEXR_OutputFile, num_scan_lines: c_int);
}


// ------------------------------------------------------------------------------
// InputFile
#[repr(C)]
pub struct CEXR_InputFile {
    header: CEXR_Header,
    input_file: *mut c_void,
}

extern "C" {
    pub fn CEXR_InputFile_new(file_name: *const c_char, num_threads: c_int) -> CEXR_InputFile;
    pub fn CEXR_InputFile_delete(input_file: *mut CEXR_InputFile);
    pub fn CEXR_InputFile_header(input_file: *const CEXR_InputFile) -> *const CEXR_Header;
    pub fn CEXR_InputFile_version(input_file: *const CEXR_InputFile) -> c_int;
    pub fn CEXR_InputFile_set_frame_buffer(input_file: *mut CEXR_InputFile, frame_buffer: *mut CEXR_FrameBuffer);
    pub fn CEXR_InputFile_is_complete(input_file: *const CEXR_InputFile) -> c_int;
    pub fn CEXR_InputFile_read_pixels(input_file: *mut CEXR_InputFile, scanline_1: c_int, scanline_2: c_int);
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let _ = unsafe { CEXR_FrameBuffer_new() };
    }
}
