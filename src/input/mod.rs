use std;
use std::collections::BTreeMap;
use std::ffi::{CString, CStr};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr;

use libc::c_char;

use openexr_sys::*;

use cexr_type_aliases::*;
use error::*;
use frame_buffer::FrameBuffer;


#[allow(dead_code)]
pub struct InputFile<'a> {
    handle: *mut CEXR_InputFile,
    channel_list: BTreeMap<String, Channel>,
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
                   channel_list: BTreeMap::new(),
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
                   channel_list: BTreeMap::new(),
                   istream: Some(istream),
                   _phantom: PhantomData,
               })
        }
    }

    pub fn read_pixels(&self, framebuffer: &mut FrameBuffer) -> Result<()> {
        let w = self.data_window();
        if (w.max.x - w.min.x) as usize != framebuffer.dimensions().0 - 1 ||
           (w.max.y - w.min.y) as usize != framebuffer.dimensions().1 - 1 {
            panic!("framebuffer size {}x{} does not match input file dimensions {}x{}",
                   framebuffer.dimensions().0,
                   framebuffer.dimensions().1,
                   w.max.x - w.min.x,
                   w.max.y - w.min.y)
        }
        unsafe { CEXR_InputFile_set_framebuffer(self.handle, framebuffer.handle_mut()) };
        let mut error_out = ptr::null();
        let error =
            unsafe { CEXR_InputFile_read_pixels(self.handle, w.min.y, w.max.y, &mut error_out) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(())
        }
    }

    pub fn data_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_data_window(CEXR_InputFile_header(self.handle)) }
    }

    pub fn display_window(&self) -> &Box2i {
        unsafe { &*CEXR_Header_display_window(CEXR_InputFile_header(self.handle)) }
    }

    pub fn channels<'b>(&'b self) -> ChannelIter<'b> {
        ChannelIter {
            iterator: unsafe { CEXR_Header_channel_list_iter(CEXR_InputFile_header(self.handle)) },
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }
}

impl<'a> Drop for InputFile<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_InputFile_delete(self.handle) };
    }
}

pub struct ChannelIter<'a> {
    iterator: *mut CEXR_ChannelListIter,
    _phantom_1: PhantomData<CEXR_ChannelListIter>,
    _phantom_2: PhantomData<&'a InputFile<'a>>,
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

impl<'a> ChannelIter<'a> {}

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
