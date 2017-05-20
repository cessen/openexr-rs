use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use openexr_sys::*;

use error::*;
use frame_buffer::FrameBuffer;
use Header;


pub struct ScanlineOutputFile {
    handle: *mut CEXR_OutputFile,
    header_ref: Header,
    _phantom: PhantomData<CEXR_OutputFile>,
}

impl ScanlineOutputFile {
    pub fn new(path: &Path, header: &Header) -> Result<ScanlineOutputFile> {
        let c_path = CString::new(path.to_str()
                                      .expect("non-unicode path handling is unimplemented")
                                      .as_bytes())
                .unwrap();
        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error = unsafe {
            // NOTE: we don't need to keep a copy of the header, because this
            // function makes a deep copy that is stored in the CEXR_OutputFile.
            CEXR_OutputFile_from_file(c_path.as_ptr(), header.handle, 1, &mut out, &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(ScanlineOutputFile {
                   handle: out,
                   header_ref: Header {
                       // NOTE: We're casting to *mut here to satisfy the
                       // field's type, but importantly we only return a
                       // const & of the Header so it retains const semantics.
                       handle: unsafe { CEXR_OutputFile_header(out) } as *mut CEXR_Header,
                       owned: false,
                       _phantom: PhantomData,
                   },
                   _phantom: PhantomData,
               })
        }
    }

    pub fn write_pixels(&mut self, framebuffer: &mut FrameBuffer) -> Result<()> {
        let w = self.header().data_window();
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

    pub fn header(&self) -> &Header {
        &self.header_ref
    }
}

impl Drop for ScanlineOutputFile {
    fn drop(&mut self) {
        unsafe { CEXR_OutputFile_delete(self.handle) };
    }
}
