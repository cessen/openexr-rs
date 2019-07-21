//! I/O compression/decompression threading control.
//!
//! This module provides functions for enabling/disabling and managing the
//! global thread pool of OpenEXR.  Importantly, if the thread pool is enabled,
//! OpenEXR will use the same thread pool for all OpenEXR reading/writing, which
//! can sometimes have unexpected performance implications for applications that
//! are already multithreaded themselves.
//!
//! By default, the thread pool is disabled.
//!
//! Please see the
//! [OpenEXR C++ library documentation](https://www.openexr.com/documentation/ReadingAndWritingImageFiles.pdf)
//! for more details.

use error::{Error, Result};

/// Sets the number of worker threads to use for compression/decompression.
///
/// If set to `0`, the thread pool is disabled and all OpenEXR calls will run
/// on their calling thread.
pub fn set_global_thread_count(thread_count: usize) -> Result<()> {
    if thread_count > ::std::os::raw::c_int::max_value() as usize {
        return Err(Error::Generic(String::from(
            "The number of threads is too high",
        )));
    }

    let mut error_out = ::std::ptr::null();

    let error = unsafe {
        openexr_sys::CEXR_set_global_thread_count(
            thread_count as ::std::os::raw::c_int,
            &mut error_out,
        )
    };
    if error != 0 {
        Err(Error::take(error_out))
    } else {
        Ok(())
    }
}

#[test]
fn test_set_global_thread_count() {
    assert!(set_global_thread_count(4).is_ok());
    assert!(set_global_thread_count(::std::os::raw::c_int::max_value() as usize + 1).is_err());
}
