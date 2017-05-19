use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr;

use libc::c_char;

use openexr_sys::*;

use cexr_type_aliases::*;
use error::*;
use frame_buffer::FrameBuffer;


pub struct ScanlineWriter {
    handle: *mut CEXR_OutputFile,
    _phantom: PhantomData<CEXR_OutputFile>,
}

impl ScanlineWriter {
    // This shouldn't be used outside of this crate, but due to
    // https://github.com/rust-lang/rfcs/pull/1422 not being stable
    // yet (should land in Rust 1.18), just hide from public
    // documentation for now.
    // TODO: once Rust 1.18 comes out, use `pub(super)`.
    pub fn new(path: *const c_char, header: *const CEXR_Header) -> Result<ScanlineWriter> {
        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error = unsafe {
            // NOTE: we don't need to keep a copy of the header, because this
            // function makes a deep copy that is stored in the CEXR_OutputFile.
            CEXR_OutputFile_from_file(path, header, 1, &mut out, &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(ScanlineWriter {
                   handle: out,
                   _phantom: PhantomData,
               })
        }
    }

    pub fn write_pixels(&mut self, framebuffer: &mut FrameBuffer) -> Result<()> {
        let w = self.data_window();
        if (w.max.x - w.min.x) as usize != framebuffer.dimensions().0 - 1 ||
           (w.max.y - w.min.y) as usize != framebuffer.dimensions().1 - 1 {
            panic!("framebuffer size {}x{} does not match output file dimensions {}x{}",
                   framebuffer.dimensions().0,
                   framebuffer.dimensions().1,
                   w.max.x - w.min.x,
                   w.max.y - w.min.y)
        }
        unsafe { CEXR_OutputFile_set_framebuffer(self.handle, framebuffer.handle()) };
        let mut error_out = ptr::null();
        let error = unsafe {
            CEXR_OutputFile_write_pixels(self.handle,
                                         framebuffer.dimensions().1 as i32,
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

impl Drop for ScanlineWriter {
    fn drop(&mut self) {
        unsafe { CEXR_OutputFile_delete(self.handle) };
    }
}
