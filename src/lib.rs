extern crate libc;
extern crate openexr_sys;

use std::{mem, slice, iter, ptr, error, fmt};
use std::path::Path;
use std::collections::HashMap;
use std::ffi::{CString, CStr};

use libc::{c_char, c_int, c_float};

use openexr_sys as cexr;

pub use openexr_sys::CEXR_PixelType as PixelType;
pub use openexr_sys::CEXR_CompressionMethod as CompressionMethod;
pub use openexr_sys::CEXR_LineOrder as LineOrder;

#[derive(Debug, Clone)]
pub enum Error {
    Generic(String)
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            Generic(ref x) => x,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            Generic(ref x) => f.pad(x),
        }
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

// ------------------------------------------------------------------------------

pub struct Box2i {
    pub min: (i32, i32),
    pub max: (i32, i32),
}


// ------------------------------------------------------------------------------

/// A trait for types that can be bit-for-bit copied as OpenEXR pixel data with
/// valid semantics.
pub unsafe trait EXRPixelData: Copy + Into<f64> {
    fn exr_pixel_data_type() -> PixelType;
}

unsafe impl EXRPixelData for u32 {
    fn exr_pixel_data_type() -> PixelType {
        PixelType::U32
    }
}

unsafe impl EXRPixelData for f32 {
    fn exr_pixel_data_type() -> PixelType {
        PixelType::F32
    }
}


// ------------------------------------------------------------------------------

/// A trait for types that can be interpreted as a set of pixel channels.
pub unsafe trait EXRPixelStruct: Copy {
    /// Returns the number if channels in the data
    fn channel_count() -> usize;

    /// Returns an array of the types and byte offsets of the channels in the data
    fn channel_descriptions() -> &'static [(PixelType, usize)];
}

unsafe impl EXRPixelStruct for (f32, f32) {
    fn channel_count() -> usize {
        2
    }

    fn channel_descriptions() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 2] = [(PixelType::F32, 0), (PixelType::F32, 4)];
        &TYPES
    }
}

unsafe impl EXRPixelStruct for (f32, f32, f32) {
    fn channel_count() -> usize {
        3
    }

    fn channel_descriptions() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 3] =
            [(PixelType::F32, 0), (PixelType::F32, 4), (PixelType::F32, 8)];
        &TYPES
    }
}

unsafe impl EXRPixelStruct for (f32, f32, f32, f32) {
    fn channel_count() -> usize {
        4
    }

    fn channel_descriptions() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 4] =
            [(PixelType::F32, 0), (PixelType::F32, 4), (PixelType::F32, 8), (PixelType::F32, 12)];
        &TYPES
    }
}

// ------------------------------------------------------------------------------

/// Describes a channel of an image (e.g. the red channel, alpha channel, an
/// arbitrary data channel) as stored on disk.  It is used for communication in
/// the API's, and doesn't actually contain any image data itself.
#[derive(Copy, Clone)]
pub struct Channel {
    pub pixel_type: PixelType,
    pub subsampling: (u32, u32), /* If set to 1, every pixel, if set to 2, every
                                  * other pixel, if 3, every third, etc. */
    pub p_linear: bool, /* Hint to lossy compression methods that indicates whether
                         * human perception of the quantity represented by this channel
                         * is closer to linear or closer to logarithmic. */
}

impl Channel {
    pub fn with_type(pixel_type: PixelType) -> Channel {
        Channel {
            pixel_type: pixel_type,
            subsampling: (1, 1),
            p_linear: true,
        }
    }
}


/// Describes a channel of an image as stored in memory.  It is used for
/// communication in the API's, and doesn't actually contain any image
/// data itself.
#[derive(Copy, Clone)]
pub struct SliceDescription {
    pub pixel_type: PixelType,
    pub subsampling: (u32, u32),
    pub start: usize,
    pub stride: (usize, usize),
    pub tile_coords: (bool, bool),
}


// ------------------------------------------------------------------------------

/// Points to and describes the in-memory storage for all channels of an
/// EXR image.  This is passed to readers and writers so they know where to
/// read/write image data in memory.
pub struct FrameBuffer<'a> {
    dimensions: (usize, usize),
    channels: HashMap<&'a str, (usize, SliceDescription, f64)>,
    buffers: Vec<&'a mut [u8]>,
}

impl<'a> FrameBuffer<'a> {
    pub fn new(width: usize, height: usize) -> FrameBuffer<'a> {
        FrameBuffer {
            dimensions: (width, height),
            channels: HashMap::new(),
            buffers: Vec::new(),
        }
    }

    pub fn add_slice<T: EXRPixelData>(&mut self, data: &'a mut [T], name: &'a str, default: T) {
        if data.len() < (self.dimensions.0 * self.dimensions.1) {
            panic!("Attempted to add too small slice to FrameBuffer.");
        }

        self.add_interleaved_slice(data, &[(name, default)]);
    }

    pub fn add_interleaved_slice<T: EXRPixelData>(&mut self,
                                                  data: &'a mut [T],
                                                  channels: &[(&'a str, T)]) {
        if channels.len() == 0 {
            panic!("Attempted to add slice without channels to FrameBuffer.");
        }
        if (data.len() / channels.len()) < (self.dimensions.0 * self.dimensions.1) {
            panic!("Attempted to add too small slice to FrameBuffer.");
        }

        // Insert channels
        let width = self.dimensions.0;
        for (i, &(name, default)) in channels.iter().enumerate() {
            self.channels.insert(name,
                                 (self.buffers.len(),
                                  SliceDescription {
                                     pixel_type: T::exr_pixel_data_type(),
                                     subsampling: (1, 1),
                                     start: mem::size_of::<T>() * i,
                                     stride: (mem::size_of::<T>(),
                                              mem::size_of::<T>() * width * channels.len()),
                                     tile_coords: (false, false),
                                 },
                                  default.into()));
        }

        // Add buffer
        let p = data.as_mut_ptr();
        let l = data.len() * mem::size_of::<T>();
        let pd = unsafe { slice::from_raw_parts_mut(p as *mut u8, l) };
        self.buffers.push(pd);
    }

    pub fn add_structured_slice<T: EXRPixelStruct>(&mut self,
                                                   data: &'a mut [T],
                                                   channels: &[(&'a str, f64)]) {
        unsafe {
            self.add_structured_slice_unsafe(data, channels, T::channel_descriptions());
        }
    }

    pub unsafe fn add_structured_slice_unsafe<T: EXRPixelStruct>(&mut self,
                                                                 data: &'a mut [T],
                                                                 channels: &[(&'a str, f64)],
                                                                 channel_descriptions: &[(PixelType,
                                                                                          usize)]) {
        if data.len() < (self.dimensions.0 * self.dimensions.1) {
            panic!("Attempted to add too small slice to FrameBuffer.");
        }
        if channels.len() == 0 {
            panic!("Attempted to add slice without channels to FrameBuffer.");
        }
        if channels.len() != channel_descriptions.len() {
            panic!("Number of channels doesn't match number of channel descriptions.");
        }
        for &(pixel_type, byte_offset) in channel_descriptions.iter() {
            if (pixel_type.data_size() + byte_offset) > mem::size_of::<T>() {
                panic!("Structured data description violates data bounds of type.");
            }
        }

        // Insert channels
        let width = self.dimensions.0;
        for (&(name, default), &(pixel_type, byte_offset)) in
            iter::Iterator::zip(channels.iter(), channel_descriptions.iter()) {
            self.channels.insert(name,
                                 (self.buffers.len(),
                                  SliceDescription {
                                     pixel_type: pixel_type,
                                     subsampling: (1, 1),
                                     start: byte_offset,
                                     stride: (mem::size_of::<T>(), mem::size_of::<T>() * width),
                                     tile_coords: (false, false),
                                 },
                                  default.into()));
        }

        // Add buffer
        let p = data.as_mut_ptr();
        let l = data.len() * mem::size_of::<T>();
        let pd = slice::from_raw_parts_mut(p as *mut u8, l);
        self.buffers.push(pd);
    }

    pub unsafe fn add_raw_slice(&mut self,
                                data: &'a mut [u8],
                                descriptions: &[(&'a str, SliceDescription, f64)]) {
        // Make sure we're not creating any duplicate channels.
        // We check this ahead of time instead of in the same loop as inserting the channels
        // so that the FrameBuffer state remains valid.
        // TODO: more error checking, specifically checking to make sure 'data' is large
        // enough for all of the described slices.
        for &(name, _, _) in descriptions {
            if self.channels.contains_key(name) {
                // TODO: return an Err instead of a panicing
                panic!("Cannot have two of the same channel name in a FrameBuffer.");
            }
        }

        // Insert channels
        for &(name, desc, default) in descriptions {
            self.channels.insert(name, (self.buffers.len(), desc, default));
        }

        // Add buffer
        self.buffers.push(data);
    }
}


// ------------------------------------------------------------------------------

/// A builder for an exr writer.  Once everything is set up, use open() to create
/// the final EXRWriter.
pub struct ExrWriterBuilder<'a> {
    path: &'a Path,
    display_window: Box2i,
    data_window: Box2i,
    pixel_aspect_ratio: f32,
    screen_window_center: (f32, f32),
    screen_window_width: f32,
    line_order: LineOrder,
    compression_method: CompressionMethod,
    channels: HashMap<String, Channel>,
}

impl<'a> ExrWriterBuilder<'a> {
    pub fn new(path: &'a Path) -> ExrWriterBuilder<'a> {
        ExrWriterBuilder {
            path: path,
            display_window: Box2i {
                min: (0, 0),
                max: (0, 0),
            },
            data_window: Box2i {
                min: (0, 0),
                max: (0, 0),
            },
            pixel_aspect_ratio: 1.0,
            screen_window_center: (0.0, 0.0),
            screen_window_width: 1.0,
            line_order: LineOrder::IncreasingY,
            compression_method: CompressionMethod::ZIP,
            channels: HashMap::new(),
        }
    }

    pub fn display_window(mut self, min: (i32, i32), max: (i32, i32)) -> ExrWriterBuilder<'a> {
        self.display_window = Box2i {
            min: min,
            max: max,
        };
        self
    }

    pub fn data_window(mut self, min: (i32, i32), max: (i32, i32)) -> ExrWriterBuilder<'a> {
        self.data_window = Box2i {
            min: min,
            max: max,
        };
        self
    }

    pub fn pixel_aspect_ratio(mut self, par: f32) -> ExrWriterBuilder<'a> {
        self.pixel_aspect_ratio = par;
        self
    }

    pub fn screen_window_center(mut self, swc: (f32, f32)) -> ExrWriterBuilder<'a> {
        self.screen_window_center = swc;
        self
    }

    pub fn screen_window_width(mut self, sww: f32) -> ExrWriterBuilder<'a> {
        self.screen_window_width = sww;
        self
    }

    pub fn line_order(mut self, lo: LineOrder) -> ExrWriterBuilder<'a> {
        self.line_order = lo;
        self
    }

    pub fn compression_method(mut self, cm: CompressionMethod) -> ExrWriterBuilder<'a> {
        self.compression_method = cm;
        self
    }

    pub fn insert_channel(mut self, name: &str, channel: Channel) -> ExrWriterBuilder<'a> {
        self.channels.insert(name.to_string(), channel);
        self
    }

    pub fn open(self) -> ExrWriter {
        // Build the header
        let header = {
            let mut header = unsafe {
                cexr::CEXR_Header_new(self.display_window.min.0 as c_int,
                                      self.display_window.min.1 as c_int,
                                      self.display_window.max.0 as c_int,
                                      self.display_window.max.1 as c_int,
                                      self.data_window.min.0 as c_int,
                                      self.data_window.min.1 as c_int,
                                      self.data_window.max.0 as c_int,
                                      self.data_window.max.1 as c_int,
                                      self.pixel_aspect_ratio as c_float,
                                      self.screen_window_center.0,
                                      self.screen_window_center.1,
                                      self.screen_window_width,
                                      self.line_order,
                                      self.compression_method)
            };
            for (name, channel) in &self.channels {
                let n = CString::new(name.as_bytes()).unwrap();
                let c = cexr::CEXR_Channel {
                    pixel_type: channel.pixel_type,
                    x_sampling: channel.subsampling.0 as c_int,
                    y_sampling: channel.subsampling.1 as c_int,
                    p_linear: if channel.p_linear {
                        1 as c_int
                    } else {
                        0 as c_int
                    },
                };
                unsafe { cexr::CEXR_Header_insert_channel(&mut header, n.as_ptr(), c) };
            }
            header
        };



        ExrWriter {
            handle: unsafe {
                cexr::CEXR_OutputFile_new(CString::new(self.path.to_str().unwrap().as_bytes())
                                              .unwrap()
                                              .as_ptr(),
                                          &header,
                                          1)
            },
        }
    }
}


// ------------------------------------------------------------------------------
pub struct ExrWriter {
    handle: cexr::CEXR_OutputFile,
}

impl Drop for ExrWriter {
    fn drop(&mut self) {
        unsafe { cexr::CEXR_OutputFile_delete(&mut self.handle) };
    }
}

impl ExrWriter {
    pub fn write_pixels(&mut self, frame_buffer: &mut FrameBuffer) {
        // Build the C frame buffer from the given frame buffer.
        let mut cexr_fb = {
            let mut cexr_fb = unsafe { cexr::CEXR_FrameBuffer_new() };
            for (&name, &(buf_index, desc, default)) in frame_buffer.channels.iter() {
                let n = CString::new(name.as_bytes()).unwrap();
                let buf_ptr = unsafe {
                    frame_buffer.buffers[buf_index].as_mut_ptr().offset(desc.start as isize)
                } as *mut c_char;
                unsafe {
                    cexr::CEXR_FrameBuffer_insert_slice(&mut cexr_fb,
                                                        n.as_ptr(),
                                                        desc.pixel_type,
                                                        buf_ptr,
                                                        desc.stride.0,
                                                        desc.stride.1,
                                                        desc.subsampling.0 as i32,
                                                        desc.subsampling.1 as i32,
                                                        default,
                                                        desc.tile_coords.0 as i32,
                                                        desc.tile_coords.1 as i32)
                };
            }
            cexr_fb
        };

        // Set the C frame buffer.
        unsafe { cexr::CEXR_OutputFile_set_frame_buffer(&mut self.handle, &mut cexr_fb) };

        // Write the pixel data.
        unsafe {
            cexr::CEXR_OutputFile_write_pixels(&mut self.handle, frame_buffer.dimensions.1 as i32)
        };

        // Destroy the C framebuffer
        unsafe { cexr::CEXR_FrameBuffer_delete(&mut cexr_fb) };
    }
}

// ------------------------------------------------------------------------------
pub struct ExrReader {
    handle: cexr::CEXR_InputFile,
}

impl ExrReader {
    pub fn new(path: &Path) -> Result<Self> {
        let c_path = CString::new(path.to_str().unwrap().as_bytes()).unwrap();
        let mut error_msg = ptr::null();
        let mut out = unsafe { mem::uninitialized() };
        let error = unsafe { cexr::CEXR_InputFile_new(c_path.as_ptr(), 1, &mut out, &mut error_msg) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_msg) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(ExrReader {
                handle: out,
            })
        }
    }

    pub fn read_pixels(&mut self, frame_buffer: &mut FrameBuffer) -> Result<()> {
        // Build the C frame buffer from the given frame buffer.
        let mut cexr_fb = {
            let mut cexr_fb = unsafe { cexr::CEXR_FrameBuffer_new() };
            for (&name, &(buf_index, desc, default)) in frame_buffer.channels.iter() {
                let n = CString::new(name.as_bytes()).unwrap();
                let buf_ptr = unsafe {
                    frame_buffer.buffers[buf_index].as_mut_ptr().offset(desc.start as isize)
                } as *mut c_char;
                unsafe {
                    cexr::CEXR_FrameBuffer_insert_slice(&mut cexr_fb,
                                                        n.as_ptr(),
                                                        desc.pixel_type,
                                                        buf_ptr,
                                                        desc.stride.0,
                                                        desc.stride.1,
                                                        desc.subsampling.0 as i32,
                                                        desc.subsampling.1 as i32,
                                                        default,
                                                        desc.tile_coords.0 as i32,
                                                        desc.tile_coords.1 as i32)
                };
            }
            cexr_fb
        };

        let mut error_msg = ptr::null();
        let error = unsafe {
            // Set the C frame buffer.
            cexr::CEXR_InputFile_set_frame_buffer(&mut self.handle, &mut cexr_fb);

            // Read the pixel data.
            let err = cexr::CEXR_InputFile_read_pixels(&mut self.handle, 0, (frame_buffer.dimensions.1 - 1) as i32, &mut error_msg);

            // Destroy the C framebuffer
            cexr::CEXR_FrameBuffer_delete(&mut cexr_fb);

            err
        };
        if error != 0 {
            let err = unsafe { CStr::from_ptr(error_msg) };
            Err(Error::Generic(err.to_string_lossy().into_owned()))
        } else {
            Ok(())
        }
    }
}

impl Drop for ExrReader {
    fn drop(&mut self) {
        unsafe { cexr::CEXR_InputFile_delete(&mut self.handle) };
    }
}


// ------------------------------------------------------------------------------
