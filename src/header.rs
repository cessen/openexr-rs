//! Header and related types.

use std::{self, slice, ptr};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;

use openexr_sys::*;

use cexr_type_aliases::*;
use error::{Error, Result};
use frame_buffer::{FrameBuffer, FrameBufferMut};
use libc::c_int;

pub use cexr_type_aliases::{Channel, Compression, LineOrder};

/// Represents an OpenEXR file header.
///
/// The file header describes the properties of the image, such as image
/// resolution, the channels it contains, custom attributes, etc.  It is used
/// both for fetching information about a loaded EXR file and for defining the
/// properties of a file to be written.
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
    pub(crate) handle: *mut CEXR_Header,
    pub(crate) owned: bool,
    pub(crate) _phantom: PhantomData<CEXR_Header>,
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
            unsafe {
                CEXR_Header_new(
                    &display_window,
                    &data_window,
                    pixel_aspect_ratio,
                    &screen_window_center,
                    screen_window_width,
                    line_order,
                    compression,
                )
            }
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
        assert!(window.min.x < window.max.x);
        assert!(window.min.y < window.max.y);
        unsafe {
            CEXR_Header_set_display_window(self.handle, window);
        }
        self
    }

    /// Sets the data window.
    ///
    /// For simple use-cases, it's better to use `set_resolution()` instead.
    pub fn set_data_window(&mut self, window: Box2i) -> &mut Self {
        assert!(window.min.x < window.max.x);
        assert!(window.min.y < window.max.y);
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
            CEXR_Header_set_screen_window_center(
                self.handle,
                CEXR_V2f {
                    x: center.0,
                    y: center.1,
                },
            );
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
        self.add_channel_detailed(
            name,
            Channel {
                pixel_type: pixel_type,
                x_sampling: 1,
                y_sampling: 1,
                p_linear: true,
            },
        )
    }

    /// Adds a channel, specifying full details.
    pub fn add_channel_detailed(&mut self, name: &str, channel: Channel) -> &mut Self {
        let cname = CString::new(name.as_bytes()).unwrap();
        unsafe { CEXR_Header_insert_channel(self.handle, cname.as_ptr(), channel) };
        self
    }

    /// Convenience method for the origin (min coordinate) of the
    /// data window.
    pub fn data_origin(&self) -> (i32, i32) {
        let window = self.data_window();
        (window.min.x, window.min.y)
    }

    /// Convenience method for the dimensions of the data window.
    pub fn data_dimensions(&self) -> (u32, u32) {
        let window = self.data_window();
        (
            (window.max.x - window.min.x + 1) as u32,
            (window.max.y - window.min.y + 1) as u32,
        )
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
    pub fn channels(&self) -> ChannelIter {
        ChannelIter {
            iterator: unsafe { CEXR_Header_channel_list_iter(self.handle) },
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    /// Access channels by name.
    pub fn get_channel<'a>(&'a self, name: &str) -> Option<&'a Channel> {
        let c_name = CString::new(name.as_bytes()).unwrap();
        let out = unsafe { CEXR_Header_get_channel(self.handle, c_name.as_ptr()) };
        if !out.is_null() {
            Some(unsafe { &(*out) })
        } else {
            None
        }
    }

    /// Determine whether this header describes an environment map, and if so, what type
    pub fn envmap(&self) -> Option<Envmap> {
        if unsafe { CEXR_Header_has_envmap(self.handle) } {
            match unsafe { CEXR_Header_envmap(self.handle) } {
                0 => Some(Envmap::LatLong),
                1 => Some(Envmap::Cube),
                _ => None,
            }
        } else {
            None
        }
    }

    /// Declare whether this header represents an environment map
    pub fn set_envmap(&mut self, envmap: Option<Envmap>) -> &mut Self {
        if let Some(x) = envmap {
            unsafe { CEXR_Header_set_envmap(self.handle, x as c_int) }
        } else {
            unsafe { CEXR_Header_erase_attribute(self.handle, b"envmap\0".as_ptr() as *const _) }
        }
        self
    }

    /// Access the list of view names, if any
    pub fn multiview(&self) -> Option<impl Iterator<Item=&str>> {
        if !unsafe { CEXR_Header_has_multiview(self.handle) } {
            return None;
        }
        let n = unsafe { CEXR_Header_multiview(self.handle, ptr::null_mut()) };
        let mut v = vec![CEXR_Slice { ptr: ptr::null_mut(), len: 0 }; n];
        unsafe { CEXR_Header_multiview(self.handle, v.as_mut_ptr()) };
        // We ignore non-UTF-8 view names because we assume all channel names are UTF-8
        let v = v.into_iter()
            .filter_map(|slice| {
                let bytes = unsafe { slice::from_raw_parts(slice.ptr as *const u8, slice.len) };
                std::str::from_utf8(&bytes).ok()
            });
        Some(v)
    }

    /// Set the list of view names
    pub fn set_multiview(&mut self, views: Option<&[&str]>) -> &mut Self {
        if let Some(x) = views {
            let slices = x.iter().map(|n| CEXR_Slice { ptr: n.as_ptr() as _, len: n.len() }).collect::<Vec<_>>();
            unsafe { CEXR_Header_set_multiview(self.handle, slices.as_ptr(), slices.len()) };
        } else {
            unsafe { CEXR_Header_erase_attribute(self.handle, b"multiView\0".as_ptr() as *const _) }
        }
        self
    }

    pub(crate) fn validate_framebuffer_for_output(&self, framebuffer: &FrameBuffer) -> Result<()> {
        for chan in self.channels() {
            let (name, h_channel) = chan?;
            if let Some(fb_channel) = framebuffer._get_channel(name) {
                Header::validate_channel(name, &h_channel, &fb_channel)?;
            } else {
                return Err(Error::Generic(format!(
                    "FrameBuffer is missing \
                     channel '{}' expected by Header",
                    name
                )));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_framebuffer_for_input(
        &self,
        framebuffer: &FrameBufferMut,
    ) -> Result<()> {
        for chan in self.channels() {
            let (name, h_channel) = chan?;
            if let Some(fb_channel) = framebuffer._get_channel(name) {
                Header::validate_channel(name, &h_channel, &fb_channel)?;
            }
        }
        Ok(())
    }

    /// Utility function to create a Box2i specifying its origin (bottom left) and size
    pub fn box2i(x: i32, y: i32, width: u32, height: u32) -> Box2i {
        Box2i {
            min: CEXR_V2i { x, y },
            max: CEXR_V2i {
                x: x + width as i32 - 1,
                y: y + height as i32 - 1,
            },
        }
    }

    // Factored out shared code from the validate_framebuffer_* methods above.
    fn validate_channel(name: &str, h_chan: &Channel, fb_chan: &Channel) -> Result<()> {
        if fb_chan.pixel_type != h_chan.pixel_type {
            return Err(Error::Generic(format!(
                "Header and FrameBuffer channel \
                 types don't match: '{}' is {:?} in Header and {:?} in \
                 FrameBuffer",
                name, h_chan.pixel_type, fb_chan.pixel_type
            )));
        }
        if fb_chan.x_sampling != h_chan.x_sampling || fb_chan.y_sampling != h_chan.y_sampling {
            return Err(Error::Generic(format!(
                "Header and FrameBuffer channel \
                 subsampling don't match: channel '{}' is {}x{} in Header and \
                 {}x{} in FrameBuffer",
                name, h_chan.x_sampling, h_chan.y_sampling, fb_chan.x_sampling, fb_chan.y_sampling
            )));
        }

        Ok(())
    }
}

impl Default for Header {
    fn default() -> Header {
        Header::new()
    }
}

impl Drop for Header {
    fn drop(&mut self) {
        if self.owned {
            unsafe { CEXR_Header_delete(self.handle) };
        }
    }
}

/// An iterator over the channels in a `Header`.
///
/// Yields a tuple of the name and description of each channel.
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
                Some(Err(Error::Generic(format!(
                    "Invalid channel name: {:?}",
                    cname
                ))))
            }
        } else {
            None
        }
    }
}

/// Types of environment maps
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Envmap {
    /// Latitude-longitude projection
    LatLong = 0,
    /// Cubemap
    Cube = 1,
}
