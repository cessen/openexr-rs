//! I/O compression/decompression threading control
//! 
//! Function set_global_thread_count creates a global pool of worker threads
//! inside the IlmImf library.  Ifan application program has multiple threads,
//! and those threads read or write several OpenEXR files at thesame time, then
//! the worker threads must be shared among the application threads.  By default
//! each file willattempt to use the entire worker thread pool for itself.
//! If two files are read or written simultaneously by two application threads,
//! then it is possible that all worker threads perform I/O on behalf of one of
//! the files, whileI/O for the other file is stalled.
//!
//! see https://www.openexr.com/documentation/ReadingAndWritingImageFiles.pdf

pub use error::{Error, Result};

/// Set the number of worker threads to use for compression/decompression.
///
/// This controls the maximum number of work threads that can be used to perform
/// compression,decompression while loading or writing a file. Note that the file I/O itself is
/// always performed on the calling thread. If this value is set to 0, multi-threaded is disabled
/// globally.
pub fn set_global_thread_count(thread_count: usize) -> Result<()> {
    if thread_count > ::std::os::raw::c_int::max_value() as usize {
        return Err(Error::Generic(String::from(
            "The number of threads is too high",
        )));
    }

    let error =
        unsafe { openexr_sys::CEXR_set_global_thread_count(thread_count as ::std::os::raw::c_int) };

    if error == 0 {
        Ok(())
    } else {
        Err(Error::Generic(String::from(
            "Unable to set global thread count",
        )))
    }
}
