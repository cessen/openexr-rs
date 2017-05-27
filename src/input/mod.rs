use std::ffi::CStr;
use std::io::{Read, Seek};
use std::marker::PhantomData;
use std::ptr;

use libc::c_char;

use openexr_sys::*;

use error::*;
use frame_buffer::FrameBufferMut;
use Header;
use stream_io::{read_stream, seek_stream};

/// Common input interface for all types of OpenEXR files
///
/// # Examples
/// ```rust,no_run
/// # use openexr::{InputFile, FrameBufferMut};
/// # use std::fs::File;
/// # use std::path::Path;
/// # let path = "/path/to/file.exr";
/// # let path = Path::new(&path);
/// let mut file = File::open(path).unwrap();
/// let input_file = InputFile::new(&mut file).unwrap();
/// let window = input_file.header().data_window();
/// let width = window.max.x - window.min.x + 1;
/// let height = window.max.y - window.min.y + 1;
///
/// let mut pixel_data: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; (width*height) as usize];
/// let mut fb = FrameBufferMut::new(width as usize, height as usize);
/// fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0), ("A", 0.0)], &mut pixel_data);
/// input_file.read_pixels(&mut fb).unwrap();
/// ```
#[allow(dead_code)]
pub struct InputFile<'a> {
    handle: *mut CEXR_InputFile,
    header_ref: Header,
    istream: *mut CEXR_IStream,
    _phantom_1: PhantomData<CEXR_InputFile>,
    _phantom_2: PhantomData<&'a mut ()>, // Represents the borrowed reader

    // NOTE: Because we don't know what type the reader might be, it's important
    // that this struct remains neither Sync nor Send.  Please don't implement
    // them!
}

impl<'a> InputFile<'a> {
    pub fn new<T: 'a>(reader: &mut T) -> Result<InputFile>
        where T: Read + Seek
    {
        let istream_ptr = {
            let read_ptr = read_stream::<T>;
            let seekp_ptr = seek_stream::<T>;

            let mut error_out = ptr::null();
            let mut out = ptr::null_mut();
            let error = unsafe {
                CEXR_IStream_from_reader(reader as *mut T as *mut _,
                                         Some(read_ptr),
                                         Some(seekp_ptr),
                                         &mut out,
                                         &mut error_out)
            };

            if error != 0 {
                let msg = unsafe { CStr::from_ptr(error_out) };
                return Err(Error::Generic(msg.to_string_lossy().into_owned()));
            } else {
                out
            }
        };

        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error = unsafe { CEXR_InputFile_from_stream(istream_ptr, 1, &mut out, &mut error_out) };
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
                   istream: istream_ptr,
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }

    pub fn from_slice(slice: &[u8]) -> Result<InputFile> {
        let istream_ptr = unsafe {
            CEXR_IStream_from_memory(b"in-memory data\0".as_ptr() as *const c_char,
                                     slice.as_ptr() as *mut u8 as *mut c_char,
                                     slice.len())
        };

        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error = unsafe { CEXR_InputFile_from_stream(istream_ptr, 1, &mut out, &mut error_out) };
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
                   istream: istream_ptr,
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }

    pub fn read_pixels(&self, framebuffer: &mut FrameBufferMut) -> Result<()> {
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
        unsafe { CEXR_IStream_delete(self.istream) };
    }
}
