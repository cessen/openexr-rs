use std::io::{Read, Write, Seek, SeekFrom};
use std::os::raw::{c_char, c_int, c_void};
use std::slice;

/// Returns 0 on success, 1 on system failure, and 2 on other failure.
///
/// ImfIO.h:
/// virtual bool read (char c[/*n*/], int n) = 0;
pub unsafe extern "C" fn read_stream<T: Read>(read: *mut c_void,
                                              c: *mut c_char,
                                              n: c_int,
                                              err_out: *mut c_int)
                                              -> c_int {
    let bytes = slice::from_raw_parts_mut(c as *mut u8, n as usize);
    match (*(read as *mut T)).read_exact(bytes) {
        Ok(_) => return 0,
        Err(e) => {
            if let Some(err) = e.raw_os_error() {
                *err_out = err as c_int;
                return 1;
            } else {
                *err_out = 0;
                return 2;
            }
        }
    }
}

/// Returns 0 on success, 1 on system failure, and 2 on other failure.
///
/// ImfIO.h:
/// virtual void write (const char c[/*n*/], int n) = 0;
pub unsafe extern "C" fn write_stream<T: Write>(writer: *mut c_void,
                                                c: *const c_char,
                                                n: c_int,
                                                err_out: *mut c_int)
                                                -> c_int {
    let bytes = slice::from_raw_parts(c as *const u8, n as usize);
    match (*(writer as *mut T)).write_all(bytes) {
        Ok(_) => return 0,
        Err(e) => {
            if let Some(err) = e.raw_os_error() {
                *err_out = err as c_int;
                return 1;
            } else {
                *err_out = 0;
                return 2;
            }
        }
    }
}

/// Returns 0 on success, 1 on system failure, and 2 on other failure.
///
/// ImfIO.h:
/// virtual void seekp (Int64 pos) = 0;
pub unsafe extern "C" fn seek_stream<T: Seek>(seeker: *mut c_void,
                                              pos: u64,
                                              err_out: *mut c_int)
                                              -> c_int {
    match (*(seeker as *mut T)).seek(SeekFrom::Start(pos)) {
        Ok(_) => return 0,
        Err(e) => {
            if let Some(err) = e.raw_os_error() {
                *err_out = err as c_int;
                return 1;
            } else {
                *err_out = 0;
                return 2;
            }
        }
    }
}
