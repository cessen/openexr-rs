use std::fmt::Arguments;
use std::io;
use std::io::{Read, Write, Seek, SeekFrom};
use std::os::raw::{c_char, c_int, c_void};
use std::slice;

/// Returns 0 on success and 1 on failure.
///
/// ImfIO.h:
/// virtual void write (const char c[/*n*/], int n) = 0;
pub unsafe extern "C" fn write_stream<T: Write>(writer: *mut c_void,
                                                c: *const c_char,
                                                n: c_int)
                                                -> c_int {
    let bytes = slice::from_raw_parts(c as *const u8, n as usize);
    if let Ok(_) = (*(writer as *mut T)).write_all(bytes) {
        return 0;
    } else {
        return 1;
    }
}

/// Returns 0 on success and 1 on failure.
///
/// ImfIO.h:
/// virtual void seekp (Int64 pos) = 0;
pub unsafe extern "C" fn seek_stream<T: Seek>(seeker: *mut c_void, pos: u64) -> c_int {
    if let Ok(_) = (*(seeker as *mut T)).seek(SeekFrom::Start(pos)) {
        return 0;
    } else {
        return 1;
    }
}

// ----------------------------------------------------------------

// Indicates an unused io stream in the type parameters of the various
// input/output file types.
pub struct Unused {}

#[allow(unused_variables)]
impl Read for Unused {
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

#[allow(unused_variables)]
impl Write for Unused {
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

#[allow(unused_variables)]
impl Seek for Unused {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        unimplemented!()
    }
}
