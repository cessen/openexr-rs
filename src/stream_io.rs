use std::io::{Write, Seek, SeekFrom};
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
