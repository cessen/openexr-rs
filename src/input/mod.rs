use std::ffi::{CString, CStr};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use libc::c_char;

use openexr_sys::*;

use error::*;
use frame_buffer::FrameBuffer;
use Header;


#[allow(dead_code)]
pub struct InputFile<'a> {
    handle: *mut CEXR_InputFile,
    header_ref: Header,
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
                   header_ref: Header {
                       // NOTE: We're casting to *mut here to satisfy the
                       // field's type, but importantly we only return a
                       // const & of the Header so it retains const semantics.
                       handle: unsafe { CEXR_InputFile_header(out) } as *mut CEXR_Header,
                       owned: false,
                       _phantom: PhantomData,
                   },
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
                   header_ref: Header {
                       // NOTE: We're casting to *mut here to satisfy the
                       // field's type, but importantly we only return a
                       // const & of the Header so it retains const semantics.
                       handle: unsafe { CEXR_InputFile_header(out) } as *mut CEXR_Header,
                       owned: false,
                       _phantom: PhantomData,
                   },
                   istream: Some(istream),
                   _phantom: PhantomData,
               })
        }
    }

    pub fn read_pixels(&self, framebuffer: &mut FrameBuffer) -> Result<()> {
        let w = self.header().data_window();
        if (w.max.x - w.min.x) as usize != framebuffer.dimensions().0 - 1 ||
           (w.max.y - w.min.y) as usize != framebuffer.dimensions().1 - 1 {
            panic!("framebuffer size {}x{} does not match input file dimensions {}x{}",
                   framebuffer.dimensions().0,
                   framebuffer.dimensions().1,
                   w.max.x - w.min.x,
                   w.max.y - w.min.y)
        }

        let mut error_out = ptr::null();

        let error = unsafe {
            CEXR_InputFile_set_framebuffer(self.handle, framebuffer.handle_mut(), &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            return Err(Error::Generic(msg.to_string_lossy().into_owned()));
        }

        let error =
            unsafe { CEXR_InputFile_read_pixels(self.handle, w.min.y, w.max.y, &mut error_out) };
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
