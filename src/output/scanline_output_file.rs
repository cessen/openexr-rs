use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use openexr_sys::*;

use error::*;
use frame_buffer::FrameBuffer;
use Header;


pub struct ScanlineOutputFile<'a> {
    handle: *mut CEXR_OutputFile,
    header_ref: Header,
    _phantom_1: PhantomData<CEXR_OutputFile>,
    _phantom_2: PhantomData<&'a mut [u8]>,
}

impl<'a> ScanlineOutputFile<'a> {
    pub fn new(path: &Path, header: &Header) -> Result<ScanlineOutputFile<'static>> {
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
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }

    pub fn write_pixels(&mut self, framebuffer: &FrameBuffer) -> Result<()> {
        framebuffer.validate_header_for_output(self.header())?;

        let mut error_out = ptr::null();

        let error = unsafe {
            CEXR_OutputFile_set_framebuffer(self.handle, framebuffer.handle(), &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            return Err(Error::Generic(msg.to_string_lossy().into_owned()));
        }

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

impl<'a> Drop for ScanlineOutputFile<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_OutputFile_delete(self.handle) };
    }
}
