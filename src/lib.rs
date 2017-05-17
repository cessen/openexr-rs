extern crate libc;
extern crate openexr_sys;

use std::{mem, ptr, error, fmt};
use std::path::Path;
use std::ffi::{CString, CStr};
use std::marker::PhantomData;

use libc::{c_char, c_int};

use openexr_sys::*;

pub use openexr_sys::CEXR_PixelType as PixelType;
pub use openexr_sys::CEXR_LineOrder as LineOrder;
pub use openexr_sys::CEXR_Compression as Compression;
pub use openexr_sys::CEXR_Box2i as Box2i;

#[derive(Debug, Clone)]
pub enum Error {
    Generic(String),
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

/// Types used by OpenEXR to represent a value held by a particular channel at
/// a particular point, suitable for being to directly by the decoder.
pub unsafe trait ChannelData: Copy + Into<f64> {
    fn pixel_type() -> PixelType;
}

unsafe impl ChannelData for u32 {
    fn pixel_type() -> PixelType {
        PixelType::UINT
    }
}

unsafe impl ChannelData for f32 {
    fn pixel_type() -> PixelType {
        PixelType::FLOAT
    }
}


// ------------------------------------------------------------------------------

/// Types that represent the values of an arbitrary collection of channels at
/// a particular point, suitable for being written to directly by the decoder.
pub unsafe trait PixelStruct: Copy {
    /// Returns an array of the types and byte offsets of the channels in the data
    fn channels() -> &'static [(PixelType, usize)];
}

unsafe impl PixelStruct for (f32, f32) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 2] = [(PixelType::FLOAT, 0), (PixelType::FLOAT, 4)];
        &TYPES
    }
}

unsafe impl PixelStruct for (f32, f32, f32) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 3] = [(PixelType::FLOAT, 0),
                                                 (PixelType::FLOAT, 4),
                                                 (PixelType::FLOAT, 8)];
        &TYPES
    }
}

unsafe impl PixelStruct for (f32, f32, f32, f32) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 4] = [(PixelType::FLOAT, 0),
                                                 (PixelType::FLOAT, 4),
                                                 (PixelType::FLOAT, 8),
                                                 (PixelType::FLOAT, 12)];
        &TYPES
    }
}

// ------------------------------------------------------------------------------

#[allow(dead_code)]
pub struct InputFile<'a> {
    handle: *mut CEXR_InputFile,
    istream: Option<IStream<'a>>,
    _phantom: PhantomData<CEXR_InputFile>,
}

impl<'a> InputFile<'a> {
    pub fn from_file(path: &Path) -> Result<InputFile<'static>> {
        let c_path = CString::new(path.to_str()
                                      .expect("non-unicode path handling is unimplemented")
                                      .as_bytes())
                .unwrap();
        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error =
            unsafe { CEXR_InputFile_from_file(c_path.as_ptr(), 1, &mut out, &mut error_out) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(InputFile {
                   handle: out,
                   istream: None,
                   _phantom: PhantomData,
               })
        }
    }

    pub fn from_memory(slice: &'a [u8]) -> Result<InputFile<'a>> {
        let istream = IStream::from_slice(slice);
        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error =
            unsafe { CEXR_InputFile_from_stream(istream.handle, 1, &mut out, &mut error_out) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(InputFile {
                   handle: out,
                   istream: Some(istream),
                   _phantom: PhantomData,
               })
        }
    }

    pub fn read_pixels(&self, framebuffer: &mut FrameBuffer) -> Result<()> {
        let w = self.data_window();
        if (w.max.x - w.min.x) as usize != framebuffer.dimensions.0 - 1 ||
           (w.max.y - w.min.y) as usize != framebuffer.dimensions.1 - 1 {
            panic!("framebuffer size {}x{} does not match input file dimensions {}x{}",
                   framebuffer.dimensions.0,
                   framebuffer.dimensions.1,
                   w.max.x - w.min.x,
                   w.max.y - w.min.y)
        }
        unsafe { CEXR_InputFile_set_framebuffer(self.handle, framebuffer.handle) };
        let mut error_out = ptr::null();
        let error =
            unsafe { CEXR_InputFile_read_pixels(self.handle, w.min.y, w.max.y, &mut error_out) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(())
        }
    }

    pub fn data_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_data_window(CEXR_InputFile_header(self.handle)) }
    }

    pub fn display_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_display_window(CEXR_InputFile_header(self.handle)) }
    }
}

impl<'a> Drop for InputFile<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_InputFile_delete(self.handle) };
    }
}

struct IStream<'a> {
    handle: *mut CEXR_IStream,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> IStream<'a> {
    fn from_slice(slice: &'a [u8]) -> IStream<'a> {
        IStream {
            handle: unsafe {
                CEXR_IStream_from_memory(b"in-memory data\0".as_ptr() as *const c_char,
                                         slice.as_ptr() as *mut u8 as *mut c_char,
                                         slice.len())
            },
            _phantom: PhantomData,
        }
    }
}

impl<'a> Drop for IStream<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_IStream_delete(self.handle) };
    }
}

// ------------------------------------------------------------------------------

pub struct OutputFile {
    handle: *mut CEXR_OutputFile,
    header_handle: *mut CEXR_Header,
    _phantom_1: PhantomData<CEXR_OutputFile>,
    _phantom_2: PhantomData<CEXR_Header>,
}

impl OutputFile {
    pub fn from_file(path: &Path,
                     resolution: (u32, u32),
                     channels: &[(&str, PixelType)],
                     compression: Compression)
                     -> Result<OutputFile> {
        // Create header
        let header = {
            let display_window = Box2i {
                min: CEXR_V2i { x: 0, y: 0 },
                max: CEXR_V2i {
                    x: resolution.0 as i32 - 1,
                    y: resolution.1 as i32 - 1,
                },
            };
            let data_window = display_window;
            let pixel_aspect_ratio = 1.0;
            let screen_window_center = CEXR_V2f { x: 0.0, y: 0.0 };
            let screen_window_width = 1.0;
            let line_order = LineOrder::INCREASING_Y;
            let header = unsafe {
                CEXR_Header_new(&display_window,
                                &data_window,
                                pixel_aspect_ratio,
                                &screen_window_center,
                                screen_window_width,
                                line_order,
                                compression)
            };
            for &(name, pixel_type) in channels.iter() {
                let channel_description = CEXR_Channel {
                    pixel_type: pixel_type,
                    x_sampling: 1,
                    y_sampling: 1,
                    p_linear: true,
                };
                let cname = CString::new(name.as_bytes()).unwrap();
                unsafe { CEXR_Header_insert_channel(header, cname.as_ptr(), channel_description) };
            }
            header
        };

        // Create file
        let c_path = CString::new(path.to_str()
                                      .expect("non-unicode path handling is unimplemented")
                                      .as_bytes())
                .unwrap();
        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error = unsafe {
            CEXR_OutputFile_from_file(c_path.as_ptr(), header, 1, &mut out, &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(OutputFile {
                   handle: out,
                   header_handle: header,
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }

    pub fn write_pixels(&mut self, framebuffer: &mut FrameBuffer) -> Result<()> {
        let w = self.data_window();
        if (w.max.x - w.min.x) as usize != framebuffer.dimensions.0 - 1 ||
           (w.max.y - w.min.y) as usize != framebuffer.dimensions.1 - 1 {
            panic!("framebuffer size {}x{} does not match output file dimensions {}x{}",
                   framebuffer.dimensions.0,
                   framebuffer.dimensions.1,
                   w.max.x - w.min.x,
                   w.max.y - w.min.y)
        }
        unsafe { CEXR_OutputFile_set_framebuffer(self.handle, framebuffer.handle) };
        let mut error_out = ptr::null();
        let error = unsafe {
            CEXR_OutputFile_write_pixels(self.handle,
                                         framebuffer.dimensions.1 as i32,
                                         &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(())
        }
    }

    pub fn data_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_data_window(CEXR_OutputFile_header(self.handle)) }
    }

    pub fn display_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_display_window(CEXR_OutputFile_header(self.handle)) }
    }
}

impl Drop for OutputFile {
    fn drop(&mut self) {
        unsafe { CEXR_OutputFile_delete(self.handle) };
        unsafe { CEXR_Header_delete(self.header_handle) };
    }
}

// ------------------------------------------------------------------------------

pub struct FrameBuffer<'a> {
    handle: *mut CEXR_FrameBuffer,
    dimensions: (usize, usize),
    _phantom_1: PhantomData<CEXR_FrameBuffer>,
    _phantom_2: PhantomData<&'a mut ()>,
}

impl<'a> FrameBuffer<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            handle: unsafe { CEXR_FrameBuffer_new() },
            dimensions: (width, height),
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    pub unsafe fn insert_raw(&mut self,
                             name: &str,
                             type_: PixelType,
                             base: *mut c_char,
                             stride: (usize, usize),
                             sampling: (c_int, c_int),
                             fill_value: f64,
                             tile_coords: (bool, bool)) {
        let c_name = CString::new(name).unwrap();
        CEXR_FrameBuffer_insert(self.handle,
                                c_name.as_ptr(),
                                type_,
                                base,
                                stride.0,
                                stride.1,
                                sampling.0,
                                sampling.1,
                                fill_value,
                                tile_coords.0 as c_int,
                                tile_coords.1 as c_int);
    }

    pub fn insert_channel<T: ChannelData>(&mut self, name: &str, fill: f64, data: &'a mut [T]) {
        if data.len() != self.dimensions.0 * self.dimensions.1 {
            panic!("data size of {} elements cannot back {}x{} framebuffer",
                   data.len(),
                   self.dimensions.0,
                   self.dimensions.1);
        }
        let width = self.dimensions.0;
        unsafe {
            self.insert_raw(name,
                            T::pixel_type(),
                            data.as_mut_ptr() as *mut c_char,
                            (mem::size_of::<T>(), width * mem::size_of::<T>()),
                            (1, 1),
                            fill,
                            (false, false))
        };
    }

    pub fn insert_pixels<T: PixelStruct>(&mut self, channels: &[(&str, f64)], data: &'a mut [T]) {
        if data.len() != self.dimensions.0 * self.dimensions.1 {
            panic!("data size of {} elements cannot back {}x{} framebuffer",
                   data.len(),
                   self.dimensions.0,
                   self.dimensions.1);
        }
        let width = self.dimensions.0;
        for (&(name, fill), &(ty, offset)) in channels.iter().zip(T::channels()) {
            unsafe {
                self.insert_raw(name,
                                ty,
                                (data.as_mut_ptr() as *mut c_char).offset(offset as isize),
                                (mem::size_of::<T>(), width * mem::size_of::<T>()),
                                (1, 1),
                                fill,
                                (false, false))
            };
        }
    }
}

impl<'a> Drop for FrameBuffer<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_FrameBuffer_delete(self.handle) };
    }
}
