use std::ffi::CString;
use std::marker::PhantomData;
use std::mem;

use libc::{c_char, c_int};

use openexr_sys::*;

use cexr_type_aliases::*;


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

pub struct FrameBuffer<'a> {
    _handle: *mut CEXR_FrameBuffer,
    _dimensions: (usize, usize),
    _phantom_1: PhantomData<CEXR_FrameBuffer>,
    _phantom_2: PhantomData<&'a mut [u8]>,
}

impl<'a> FrameBuffer<'a> {
    pub fn new(width: usize, height: usize) -> Self {
        FrameBuffer {
            _handle: unsafe { CEXR_FrameBuffer_new() },
            _dimensions: (width, height),
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        self._dimensions
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

    pub fn insert_channel<T: ChannelData>(&mut self, name: &str, fill: f64, data: &'a mut [T]) {
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

    pub fn insert_pixels<T: PixelStruct>(&mut self, channels: &[(&str, f64)], data: &'a mut [T]) {
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
