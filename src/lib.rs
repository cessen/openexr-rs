//! Rust bindings for the [OpenEXR](http://openexr.com) C++ library.
//!
//! OpenEXR is a bitmap image file format that can store high dynamic images
//! along with other arbitrary per-pixel data. It is used heavily in the VFX
//! and 3D animation industries.
//!
//! Although this wrapper differs a little in its API compared to the C++
//! library, it tried not to differ wildly.  Therefore the
//! [C++ OpenEXR documentation]
//! (https://github.com/openexr/openexr/tree/develop/OpenEXR/doc) is still
//! useful as an introduction and rough reference.  Moreover, the file format
//! itself is also documented there.
//!
//! # Examples
//!
//! Writing a scanline RGB file.
//!
//! ```no_run
//! use std::fs::File;
//! use std::path::Path;
//! use openexr::{FrameBuffer, Header, ScanlineOutputFile, PixelType};
//!
//! // Pixel data for a 256x256 floating point RGB image.
//! let mut pixel_data = vec![(0.82f32, 1.78f32, 0.21f32); 256 * 256];
//!
//! // Create a file to write to.  The `Header` determines the properties of the
//! // file, like resolution and what channels it has.
//! let mut file = File::create("output_file.exr").unwrap();
//! let mut output_file = ScanlineOutputFile::new(
//!     &mut file,
//!     Header::new()
//!         .set_resolution(256, 256)
//!         .add_channel("R", PixelType::FLOAT)
//!         .add_channel("G", PixelType::FLOAT)
//!         .add_channel("B", PixelType::FLOAT)).unwrap();
//!
//! // The `FrameBuffer` points to and describes pixel data in memory. In this
//! // case, it points to `pixel_data` and says it has three channels: 'R', 'G',
//! // and 'B'.  The data type of the channels is inferred from `pixel_data`'s
//! // type.
//! //
//! // The `0.0`'s are fallback values for each channel in case of missing
//! // pixel data when reading from a file.  In this example they aren't used.
//! let mut fb = FrameBuffer::new(256, 256);
//! fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
//!
//! // Write pixel data to the file.  We pass our framebuffer to it so it knows
//! // what pixel data to write.
//! output_file.write_pixels(&fb).unwrap();
//! ```
//!
//! Reading an RGB file.
//!
//! ```no_run
//! use std::fs::File;
//! use std::path::Path;
//! use openexr::{FrameBuffer, InputFile, PixelType};
//!
//! // Open the EXR file.
//! let mut file = File::open(Path::new("input_file.exr")).unwrap();
//! let input_file = InputFile::new(&mut file).unwrap();
//!
//! // Get the image dimensions, so we know how large of a buffer to make.
//! let window = input_file.header().data_window();
//! let width = window.max.x - window.min.x + 1;
//! let height = window.max.y - window.min.y + 1;
//!
//! // Make sure the channels we want exist in the file and are of the type we
//! // expect.  If you statically know the properties of the file you're
//! // loading, this isn't necessary.
//! for channel_name in ["R", "G", "B"].iter() {
//!     let channel = input_file
//!         .header()
//!         .get_channel(channel_name)
//!         .expect(&format!("Didn't find channel {}.", channel_name));
//!     assert!(channel.pixel_type == PixelType::FLOAT);
//! }
//!
//! // Space to read pixel data into.
//! let mut pixel_data = vec![(0.0f32, 0.0f32, 0.0f32); (width*height) as usize];
//!
//! // New scope because `FrameBuffer` mutably borrows `pixel_data`, so we need
//! // it to go out of scope before we can access our `pixel_data` again.
//! {
//!     // Create `FrameBuffer`.  Same drill as for output.
//!     let mut fb = FrameBuffer::new(width as usize, height as usize);
//!     fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
//!
//!     // Read in pixel data.
//!     input_file.read_pixels(&mut fb).unwrap();
//! }
//!
//! // The image data is now loaded into `pixel_data`.
//! ```


// #![warn(missing_docs)]

extern crate half;
extern crate libc;
extern crate openexr_sys;

mod cexr_type_aliases;
mod error;
mod frame_buffer;
mod input;
mod output;
mod stream_io;

use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use openexr_sys::*;

pub use cexr_type_aliases::*;
pub use error::*;
pub use frame_buffer::*;
pub use input::*;
pub use output::*;

// TODO: move Header to its own module once we can use
// `pub(crate)` on struct fields (should be in Rust 1.18).

/// File header of an OpenEXR file.
///
/// This represents the header of an OpenEXR file.  It contains attributes, channel
/// descriptions, image resolution information, etc.  It is used both for fetching
/// information about a loaded EXR file and for defining the header of a file to be
/// written.
///
/// # Examples
///
/// Creating a header for a file that will be written:
///
/// ```
/// use openexr::{Header, PixelType};
///
/// Header::new()
///     .set_resolution(1920, 1080)
///     .add_channel("R", PixelType::FLOAT)
///     .add_channel("G", PixelType::FLOAT)
///     .add_channel("B", PixelType::FLOAT);
/// ```
pub struct Header {
    handle: *mut CEXR_Header,
    owned: bool,
    _phantom: PhantomData<CEXR_Header>,
}

impl Header {
    /// Creates a new header.
    pub fn new() -> Self {
        // Create underlying C header
        let header = {
            let display_window = Box2i {
                min: CEXR_V2i { x: 0, y: 0 },
                max: CEXR_V2i { x: 1, y: 1 },
            };
            let data_window = display_window;
            let pixel_aspect_ratio = 1.0;
            let screen_window_center = CEXR_V2f { x: 0.0, y: 0.0 };
            let screen_window_width = 1.0;
            let line_order = LineOrder::INCREASING_Y;
            let compression = Compression::PIZ_COMPRESSION;
            let header = unsafe {
                CEXR_Header_new(&display_window,
                                &data_window,
                                pixel_aspect_ratio,
                                &screen_window_center,
                                screen_window_width,
                                line_order,
                                compression)
            };
            header
        };

        Self {
            handle: header,
            owned: true,
            _phantom: PhantomData,
        }
    }

    /// Sets the resolution.
    ///
    /// This is really just a shortcut for setting both the display window
    /// and data window to `(0, 0), (width-1, height-1)`.
    pub fn set_resolution(&mut self, width: u32, height: u32) -> &mut Self {
        let window = Box2i {
            min: CEXR_V2i { x: 0, y: 0 },
            max: CEXR_V2i {
                x: width as i32 - 1,
                y: height as i32 - 1,
            },
        };
        unsafe {
            CEXR_Header_set_display_window(self.handle, window);
        }
        unsafe {
            CEXR_Header_set_data_window(self.handle, window);
        }
        self
    }

    /// Sets the display window.
    ///
    /// For simple use-cases, it's better to use `set_resolution()` instead.
    pub fn set_display_window(&mut self, window: Box2i) -> &mut Self {
        unsafe {
            CEXR_Header_set_display_window(self.handle, window);
        }
        self
    }

    /// Sets the data window.
    ///
    /// For simple use-cases, it's better to use `set_resolution()` instead.
    pub fn set_data_window(&mut self, window: Box2i) -> &mut Self {
        unsafe {
            CEXR_Header_set_data_window(self.handle, window);
        }
        self
    }

    /// Sets the pixel aspect ratio.
    pub fn set_pixel_aspect_ratio(&mut self, aspect_ratio: f32) -> &mut Self {
        unsafe {
            CEXR_Header_set_pixel_aspect_ratio(self.handle, aspect_ratio);
        }
        self
    }

    /// Sets the screen window center.
    pub fn set_screen_window_center(&mut self, center: (f32, f32)) -> &mut Self {
        unsafe {
            CEXR_Header_set_screen_window_center(self.handle,
                                                 CEXR_V2f {
                                                     x: center.0,
                                                     y: center.1,
                                                 });
        }
        self
    }

    /// Sets the screen window width.
    pub fn set_screen_window_width(&mut self, width: f32) -> &mut Self {
        unsafe {
            CEXR_Header_set_screen_window_width(self.handle, width);
        }
        self
    }

    /// Sets the line order.
    pub fn set_line_order(&mut self, line_order: LineOrder) -> &mut Self {
        unsafe {
            CEXR_Header_set_line_order(self.handle, line_order);
        }
        self
    }

    /// Sets the compression mode.
    pub fn set_compression(&mut self, compression: Compression) -> &mut Self {
        unsafe {
            CEXR_Header_set_compression(self.handle, compression);
        }
        self
    }

    /// Adds a channel.
    ///
    /// This is a simplified version of `add_channel_detailed()`, using some reasonable
    /// defaults for the details.  Specifically: sampling is set to (1, 1)
    /// and p_linear is set to true.
    pub fn add_channel(&mut self, name: &str, pixel_type: PixelType) -> &mut Self {
        self.add_channel_detailed(name,
                                  Channel {
                                      pixel_type: pixel_type,
                                      x_sampling: 1,
                                      y_sampling: 1,
                                      p_linear: true,
                                  })
    }

    /// Adds a channel, specifying full details.
    pub fn add_channel_detailed(&mut self, name: &str, channel: Channel) -> &mut Self {
        let cname = CString::new(name.as_bytes()).unwrap();
        unsafe { CEXR_Header_insert_channel(self.handle, cname.as_ptr(), channel) };
        self
    }

    /// Access to the data window.
    pub fn data_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_data_window(self.handle) }
    }

    /// Access to the display window.
    pub fn display_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_display_window(self.handle) }
    }

    /// Returns an iterator over the channels in the header.
    pub fn channels<'a>(&'a self) -> ChannelIter<'a> {
        ChannelIter {
            iterator: unsafe { CEXR_Header_channel_list_iter(self.handle) },
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    /// Access channels by name.
    pub fn get_channel<'a>(&'a self, name: &str) -> Option<&'a Channel> {
        let c_name = CString::new(name.as_bytes()).unwrap();
        let mut error_out = std::ptr::null();
        let mut out = std::ptr::null();
        if unsafe {
               CEXR_Header_get_channel(self.handle, c_name.as_ptr(), &mut out, &mut error_out)
           } == 0 {
            Some(unsafe { &(*out) })
        } else {
            None
        }
    }
}

impl Drop for Header {
    fn drop(&mut self) {
        if self.owned {
            unsafe { CEXR_Header_delete(self.handle) };
        }
    }
}

pub struct ChannelIter<'a> {
    iterator: *mut CEXR_ChannelListIter,
    _phantom_1: PhantomData<CEXR_ChannelListIter>,
    _phantom_2: PhantomData<&'a Header>,
}

impl<'a> Drop for ChannelIter<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_ChannelListIter_delete(self.iterator) };
    }
}

impl<'a> Iterator for ChannelIter<'a> {
    type Item = Result<(&'a str, Channel)>;
    fn next(&mut self) -> Option<Result<(&'a str, Channel)>> {
        let mut name = unsafe { std::mem::uninitialized() };
        let mut channel = unsafe { std::mem::uninitialized() };
        if unsafe { CEXR_ChannelListIter_next(self.iterator, &mut name, &mut channel) } {
            // TODO: use CStr::from_bytes_with_nul() instead to avoid memory unsafety
            // if the string is not nul terminated.
            let cname = unsafe { CStr::from_ptr(name) };
            let str_name = cname.to_str();
            if let Ok(n) = str_name {
                Some(Ok((n, channel)))
            } else {
                Some(Err(Error::Generic(format!("Invalid channel name: {:?}", cname))))
            }
        } else {
            None
        }
    }
}
