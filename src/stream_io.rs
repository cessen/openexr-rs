use std::fmt::Arguments;
use std::io;
use std::io::{Read, Write, Seek, SeekFrom};
use std::os::raw::{c_char, c_int, c_void};
use std::slice;

/// A pointer to this is passed to the OpenEXR C++ API for writing
/// to the IO source it represents.  It hides T from the C++ API
/// and also keeps track of the cursor position, which Rust's Seek
/// trait doesn't expose.
///
/// Note: the reason we can't just pass the pointer to T directly
/// is because it could be a fat pointer to a trait object.
pub struct StreamWriter<'a, T: 'a + Write + Seek> {
    writer: &'a mut T,
    cursor_pos: usize,
}

impl<'a, T: 'a + Write + Seek> StreamWriter<'a, T> {
    pub fn new<'b>(writer: &'b mut T) -> StreamWriter<'b, T> {
        writer
            .seek(SeekFrom::Start(0))
            .expect("Couldn't seek to zero.");
        StreamWriter {
            writer: writer,
            cursor_pos: 0,
        }
    }
}

/// Returns 0 on success and 1 on failure.
///
/// ImfIO.h:
/// virtual void write (const char c[/*n*/], int n) = 0;
pub unsafe extern "C" fn write<T: Write + Seek>(stream_writer: *mut c_void,
                                                c: *const c_char,
                                                n: c_int)
                                                -> c_int {
    let stream_writer = stream_writer as *mut StreamWriter<T>;
    let bytes = slice::from_raw_parts(c as *const u8, n as usize);
    if let Ok(_) = (*stream_writer).writer.write_all(bytes) {
        (*stream_writer).cursor_pos += n as usize;
        return 0;
    } else {
        return 1;
    }
}

/// ImfIO.h:
/// virtual Int64 tellp () = 0;
pub unsafe extern "C" fn tellp<T: Write + Seek>(stream_writer: *mut c_void) -> u64 {
    let stream_writer = stream_writer as *mut StreamWriter<T>;
    (*stream_writer).cursor_pos as u64
}

/// Returns 0 on success and 1 on failure.
///
/// ImfIO.h:
/// virtual void seekp (Int64 pos) = 0;
pub unsafe extern "C" fn seekp<T: Write + Seek>(stream_writer: *mut c_void, pos: u64) -> c_int {
    let stream_writer = stream_writer as *mut StreamWriter<T>;
    if let Ok(new_pos) = (*stream_writer).writer.seek(SeekFrom::Start(pos)) {
        (*stream_writer).cursor_pos = new_pos as usize;
        return 0;
    } else {
        return 1;
    }
}

// ----------------------------------------------------------------

pub struct UnusedIOStream {}

impl Read for UnusedIOStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unimplemented!()
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        unimplemented!()
    }
    fn read_to_string(&mut self, buf: &mut String) -> io::Result<usize> {
        unimplemented!()
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        unimplemented!()
    }
    fn by_ref(&mut self) -> &mut Self {
        unimplemented!()
    }
}

impl Write for UnusedIOStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!()
    }
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        unimplemented!()
    }
    fn write_fmt(&mut self, fmt: Arguments) -> io::Result<()> {
        unimplemented!()
    }
    fn by_ref(&mut self) -> &mut Self {
        unimplemented!()
    }
}

impl Seek for UnusedIOStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        unimplemented!()
    }
}
