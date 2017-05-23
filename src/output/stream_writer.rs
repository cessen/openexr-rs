use std::os::raw::{c_char, c_int};
use std::io::{Write, Seek, SeekFrom};
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

// These functions will be passed to the OpenEXR C++ API.

/// Returns 0 on success and 1 on failure.
///
/// ImfIO.h:
/// virtual void write (const char c[/*n*/], int n) = 0;
pub extern "C" fn write<T: Write + Seek>(stream_writer: *mut StreamWriter<T>,
                                         c: *const c_char,
                                         n: c_int)
                                         -> c_int {
    let bytes = unsafe { slice::from_raw_parts(c as *const u8, n as usize) };
    if let Ok(_) = unsafe { (*stream_writer).writer.write_all(bytes) } {
        unsafe { (*stream_writer).cursor_pos += n as usize };
        return 0;
    } else {
        return 1;
    }
}

/// ImfIO.h:
/// virtual Int64 tellp () = 0;
pub extern "C" fn tellp<T: Write + Seek>(stream_writer: *mut StreamWriter<T>) -> i64 {
    unsafe { (*stream_writer).cursor_pos as i64 }
}

/// Returns 0 on success and 1 on failure.
///
/// ImfIO.h:
/// virtual void seekp (Int64 pos) = 0;
pub extern "C" fn seekp<T: Write + Seek>(stream_writer: *mut StreamWriter<T>, pos: i64) -> c_int {
    if let Ok(new_pos) = unsafe { (*stream_writer).writer.seek(SeekFrom::Start(pos as u64)) } {
        unsafe { (*stream_writer).cursor_pos = new_pos as usize };
        return 0;
    } else {
        return 1;
    }
}
