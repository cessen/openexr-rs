use std::ffi::CString;
use std::marker::PhantomData;
use std::mem;

use half::f16;
use libc::{c_char, c_int};

use openexr_sys::*;

use cexr_type_aliases::*;
use error::*;
use Header;


/// Points to and describes in-memory image data for reading and writing.
///
/// `FrameBuffer` does not store any image data itself, but instead points to
/// and describes image data elsewhere in memory. Those descriptions are then
/// used by the input and output file types to know where in memory to read
/// from and write to when writing and reading files.
pub struct FrameBuffer<'a> {
    handle: *mut CEXR_FrameBuffer,
    dimensions: (usize, usize),
    _phantom_1: PhantomData<CEXR_FrameBuffer>,
    _phantom_2: PhantomData<&'a mut [u8]>,
}

impl<'a> FrameBuffer<'a> {
    /// Creates an empty frame buffer with the given dimensions in pixels.
    ///
    /// `FrameBuffer` does not store any data, therefore its size in memory is
    /// independent of the dimensions specified here.
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            handle: unsafe { CEXR_FrameBuffer_new() },
            dimensions: (width, height),
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    /// Return the dimensions of the frame buffer.
    pub fn dimensions(&self) -> (usize, usize) {
        self.dimensions
    }

    /// Insert a single channel.
    ///
    /// The channel will be given the name `name` and when reading from a file
    /// will be filled in with the value `fill` if there isn't any pixel data
    /// for that channel in the file.
    ///
    /// `data` is the memory for the channel and should contain precisely
    /// width * height elements, where width and height are the dimensions
    /// of the `FrameBuffer`.
    pub fn insert_channel<T: PixelData>(&mut self, name: &str, fill: f64, data: &'a mut [T]) -> &mut Self {
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
        self
    }

    /// Insert multiple channels from a slice of structs or tuples.
    ///
    /// The number of channels to be inserted is determined by the
    /// implementation of the `PixelStruct` trait on `T`.  `channels` should
    /// contain that number of elements, and each element is a tuple of the
    /// channel's name and default fill value.
    ///
    /// `data` is the memory for the channel and should contain precisely
    /// width * height elements, where width and height are the dimensions
    /// of the `FrameBuffer`.
    pub fn insert_pixels<T: PixelStruct>(&mut self, channels: &[(&str, f64)], data: &'a mut [T]) -> &mut Self {
        if data.len() != self.dimensions.0 * self.dimensions.1 {
            panic!("data size of {} elements cannot back {}x{} framebuffer",
                   data.len(),
                   self.dimensions.0,
                   self.dimensions.1);
        }
        let width = self.dimensions.0;
        for (&(name, fill), (ty, offset)) in channels.iter().zip(T::channels()) {
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
        self
    }

    /// The raw method for inserting a new channel.
    ///
    /// This is very unsafe: the other methods should be preferred unless you
    /// have a special use-case.
    ///
    /// This method corresponds directly to constructing and then inserting a
    /// "Slice" in the C++ OpenEXR library.  Please see its documentation for
    /// details.
    pub unsafe fn insert_raw(&mut self,
                             name: &str,
                             type_: PixelType,
                             base: *mut c_char,
                             stride: (usize, usize),
                             sampling: (c_int, c_int),
                             fill_value: f64,
                             tile_coords: (bool, bool)) -> &mut Self {
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
        self
    }

    // These shouldn't be used outside of this crate, but due to
    // https://github.com/rust-lang/rfcs/pull/1422 not being stable
    // yet (should land in Rust 1.18), just hide from public
    // documentation for now.
    // TODO: once Rust 1.18 comes out, remove these functions and
    // just use direct field access via `pub(crate)`.
    #[doc(hidden)]
    pub fn handle(&self) -> *const CEXR_FrameBuffer {
        self.handle
    }

    #[doc(hidden)]
    pub fn handle_mut(&mut self) -> *mut CEXR_FrameBuffer {
        self.handle
    }

    // TODO: this should probably be part of Header.  It's only here
    // right now to allow access to both struct's internals, but it won't
    // have to be here for that once `pub(crate)` lands in Rust 1.18.
    pub fn validate_header_for_output(&self, header: &Header) -> Result<()> {
        let w = header.data_window();
        if (w.max.x - w.min.x) as usize != self.dimensions().0 - 1 ||
           (w.max.y - w.min.y) as usize != self.dimensions().1 - 1 {
            return Err(Error::Generic(format!("framebuffer size {}x{} does not \
                match output file dimensions {}x{}",
                                              self.dimensions().0,
                                              self.dimensions().1,
                                              w.max.x - w.min.x,
                                              w.max.y - w.min.y)));
        }

        Ok(())
    }
}

impl<'a> Drop for FrameBuffer<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_FrameBuffer_delete(self.handle) };
    }
}


/// Types that are bitwise- and semantically-identical to one of the
/// `PixelType` variants.
///
/// Implementing this trait on a type allows the type to be used directly by the
/// library to write data out to and read data in from EXR files.
///
/// NOTE: unless you really know what you're doing, you probably shouldn't
/// implement this for anything. It's already been implemented for the
/// relevant built-in Rust types.  The only exception might be implementing it
/// for alternative implementations of half floats.
pub unsafe trait PixelData {
    fn pixel_type() -> PixelType;
}

unsafe impl PixelData for u32 {
    fn pixel_type() -> PixelType {
        PixelType::UINT
    }
}

unsafe impl PixelData for f16 {
    fn pixel_type() -> PixelType {
        PixelType::HALF
    }
}

unsafe impl PixelData for f32 {
    fn pixel_type() -> PixelType {
        PixelType::FLOAT
    }
}

/// Types that contain components that are bitwise- and semantically-identical
/// to the `PixelType` variants.
///
/// The intended use of this is to allow e.g. a tuple or struct of RGB values
/// to be used directly by the library to write data out to and read data in
/// from EXR files.  This avoids having to create buffers of converted values.
///
/// # Examples
///
/// ```
/// use openexr::{PixelStruct, PixelType};
///
/// #[repr(C)]
/// #[derive(Copy, Clone)]
/// struct RGB {
///     r: f32,
///     g: f32,
///     b: f32,
/// }
///
/// unsafe impl PixelStruct for RGB {
///     fn channel_count() -> usize { 3 }
///     fn channel(i: usize) -> (PixelType, usize) {
///         [(PixelType::FLOAT, 0),
///          (PixelType::FLOAT, 4),
///          (PixelType::FLOAT, 8)][i]
///     }
/// }
/// ```
pub unsafe trait PixelStruct {
    /// Returns the number of channels in this type
    fn channel_count() -> usize;

    /// Returns the type and offset of channel `i`
    /// # Panics
    /// Will either panic or return garbage when `i >= channel_count()`.
    fn channel(i: usize) -> (PixelType, usize);

    /// Returns an iterator over the set of channels
    fn channels() -> PixelStructChannels {
        (0..Self::channel_count()).map(Self::channel)
    }
}

pub type PixelStructChannels = ::std::iter::Map<::std::ops::Range<usize>,
                                                fn(usize) -> (PixelType, usize)>;

unsafe impl<T: PixelData> PixelStruct for T {
    fn channel_count() -> usize {
        1
    }
    fn channel(_: usize) -> (PixelType, usize) {
        (T::pixel_type(), 0)
    }
}

macro_rules! offset_of {
    ($ty:ty, $field:tt) => {
        unsafe { &(*(0 as *const $ty)).$field as *const _ as usize }
    }
}

unsafe impl<A: PixelData> PixelStruct for (A,) {
    fn channel_count() -> usize {
        1
    }
    fn channel(_: usize) -> (PixelType, usize) {
        (A::pixel_type(), offset_of!(Self, 0))
    }
}

unsafe impl<A, B> PixelStruct for (A, B)
    where A: PixelData,
          B: PixelData
{
    fn channel_count() -> usize {
        2
    }
    fn channel(i: usize) -> (PixelType, usize) {
        [(A::pixel_type(), offset_of!(Self, 0)),
         (B::pixel_type(), offset_of!(Self, 1))][i]
    }
}

unsafe impl<A, B, C> PixelStruct for (A, B, C)
    where A: PixelData,
          B: PixelData,
          C: PixelData
{
    fn channel_count() -> usize {
        3
    }
    fn channel(i: usize) -> (PixelType, usize) {
        [(A::pixel_type(), offset_of!(Self, 0)),
         (B::pixel_type(), offset_of!(Self, 1)),
         (C::pixel_type(), offset_of!(Self, 2))]
            [i]
    }
}

unsafe impl<A, B, C, D> PixelStruct for (A, B, C, D)
    where A: PixelData,
          B: PixelData,
          C: PixelData,
          D: PixelData
{
    fn channel_count() -> usize {
        4
    }
    fn channel(i: usize) -> (PixelType, usize) {
        [(A::pixel_type(), offset_of!(Self, 0)),
         (B::pixel_type(), offset_of!(Self, 1)),
         (C::pixel_type(), offset_of!(Self, 2)),
         (D::pixel_type(), offset_of!(Self, 3))]
            [i]
    }
}

unsafe impl<T: PixelData> PixelStruct for [T; 1] {
    fn channel_count() -> usize {
        1
    }
    fn channel(_: usize) -> (PixelType, usize) {
        (T::pixel_type(), 0)
    }
}

unsafe impl<T: PixelData> PixelStruct for [T; 2] {
    fn channel_count() -> usize {
        2
    }
    fn channel(i: usize) -> (PixelType, usize) {
        (T::pixel_type(), i * mem::size_of::<T>())
    }
}

unsafe impl<T: PixelData> PixelStruct for [T; 3] {
    fn channel_count() -> usize {
        3
    }
    fn channel(i: usize) -> (PixelType, usize) {
        (T::pixel_type(), i * mem::size_of::<T>())
    }
}

unsafe impl<T: PixelData> PixelStruct for [T; 4] {
    fn channel_count() -> usize {
        4
    }
    fn channel(i: usize) -> (PixelType, usize) {
        (T::pixel_type(), i * mem::size_of::<T>())
    }
}
