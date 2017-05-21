use std::ffi::CString;
use std::marker::PhantomData;
use std::mem;

use half::f16;
use libc::{c_char, c_int};

use openexr_sys::*;

use cexr_type_aliases::*;


/// Points to and describes in-memory image data for reading and writing.
///
/// `FrameBuffer` does not store any image data itself, but instead points to
/// and describes image data elsewhere in memory. Those descriptions are then
/// used by the input and output file types to know where in memory to read
/// from and write to when writing and reading files.
pub struct FrameBuffer<'a> {
    _handle: *mut CEXR_FrameBuffer,
    _dimensions: (usize, usize),
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
            _handle: unsafe { CEXR_FrameBuffer_new() },
            _dimensions: (width, height),
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    /// Return the dimensions of the frame buffer.
    pub fn dimensions(&self) -> (usize, usize) {
        self._dimensions
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
    pub fn insert_channel<T: PixelData>(&mut self, name: &str, fill: f64, data: &'a mut [T]) {
        if data.len() != self._dimensions.0 * self._dimensions.1 {
            panic!("data size of {} elements cannot back {}x{} framebuffer",
                   data.len(),
                   self._dimensions.0,
                   self._dimensions.1);
        }
        let width = self._dimensions.0;
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

    /// Insert multiple channels from a slice of structs or tuples.
    ///
    /// The number of channels to be inserted is determined by the
    /// implementation of the `PixelDataStruct` trait on `T`.  `channels` should
    /// contain that number of elements, and each element is a tuple of the
    /// channel's name and default fill value.
    ///
    /// `data` is the memory for the channel and should contain precisely
    /// width * height elements, where width and height are the dimensions
    /// of the `FrameBuffer`.
    pub fn insert_pixels<T: PixelDataStruct>(&mut self,
                                             channels: &[(&str, f64)],
                                             data: &'a mut [T]) {
        if data.len() != self._dimensions.0 * self._dimensions.1 {
            panic!("data size of {} elements cannot back {}x{} framebuffer",
                   data.len(),
                   self._dimensions.0,
                   self._dimensions.1);
        }
        let width = self._dimensions.0;
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
                             tile_coords: (bool, bool)) {
        let c_name = CString::new(name).unwrap();
        CEXR_FrameBuffer_insert(self._handle,
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

    // These shouldn't be used outside of this crate, but due to
    // https://github.com/rust-lang/rfcs/pull/1422 not being stable
    // yet (should land in Rust 1.18), just hide from public
    // documentation for now.
    // TODO: once Rust 1.18 comes out, remove these functions and
    // just use direct field access via `pub(crate)`.
    #[doc(hidden)]
    pub fn handle(&self) -> *const CEXR_FrameBuffer {
        self._handle
    }

    #[doc(hidden)]
    pub fn handle_mut(&mut self) -> *mut CEXR_FrameBuffer {
        self._handle
    }
}

impl<'a> Drop for FrameBuffer<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_FrameBuffer_delete(self._handle) };
    }
}


/// Types that are bitwise- and semantically-identical to one of the
/// `PixelType` variants.
///
/// Implementing this trait on a type allows the type to be used directly
/// by the library to write data out to and read data in from EXR files.
pub unsafe trait PixelData: Copy + Into<f64> {
    /// Returns which `PixelType` variant the type is equivalent to.
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
/// use openexr::{PixelDataStruct, PixelType};
///
/// #[repr(C)]
/// #[derive(Copy, Clone)]
/// struct RGB {
///     r: f32,
///     g: f32,
///     b: f32,
/// }
///
/// unsafe impl PixelDataStruct for RGB {
///     fn channels() -> &'static [(PixelType, usize)] {
///         static TYPES: [(PixelType, usize); 3] = [(PixelType::FLOAT, 0),
///                                                  (PixelType::FLOAT, 4),
///                                                  (PixelType::FLOAT, 8)];
///         &TYPES
///     }
/// }
/// ```
pub unsafe trait PixelDataStruct: Copy {
    /// Returns an array of the types and byte offsets of the channels in the data
    fn channels() -> &'static [(PixelType, usize)];
}

// T2_F32, T3_F32, T4_F32,
include!(concat!(env!("OUT_DIR"), "/data_type_offsets.rs"));

unsafe impl PixelDataStruct for (f16, f16) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 2] = [(PixelType::FLOAT, T2_F16.0),
                                                 (PixelType::FLOAT, T2_F16.1)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for (f16, f16, f16) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 3] = [(PixelType::FLOAT, T3_F16.0),
                                                 (PixelType::FLOAT, T3_F16.1),
                                                 (PixelType::FLOAT, T3_F16.2)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for (f16, f16, f16, f16) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 4] = [(PixelType::FLOAT, T4_F16.0),
                                                 (PixelType::FLOAT, T4_F16.1),
                                                 (PixelType::FLOAT, T4_F16.2),
                                                 (PixelType::FLOAT, T4_F16.3)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for (f32, f32) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 2] = [(PixelType::FLOAT, T2_F32.0),
                                                 (PixelType::FLOAT, T2_F32.1)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for (f32, f32, f32) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 3] = [(PixelType::FLOAT, T3_F32.0),
                                                 (PixelType::FLOAT, T3_F32.1),
                                                 (PixelType::FLOAT, T3_F32.2)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for (f32, f32, f32, f32) {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 4] = [(PixelType::FLOAT, T4_F32.0),
                                                 (PixelType::FLOAT, T4_F32.1),
                                                 (PixelType::FLOAT, T4_F32.2),
                                                 (PixelType::FLOAT, T4_F32.3)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for [f32; 2] {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 2] = [(PixelType::FLOAT, 0), (PixelType::FLOAT, 4)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for [f32; 3] {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 3] = [(PixelType::FLOAT, 0),
                                                 (PixelType::FLOAT, 4),
                                                 (PixelType::FLOAT, 8)];
        &TYPES
    }
}

unsafe impl PixelDataStruct for [f32; 4] {
    fn channels() -> &'static [(PixelType, usize)] {
        static TYPES: [(PixelType, usize); 4] = [(PixelType::FLOAT, 0),
                                                 (PixelType::FLOAT, 4),
                                                 (PixelType::FLOAT, 8),
                                                 (PixelType::FLOAT, 12)];
        &TYPES
    }
}
