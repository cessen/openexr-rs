//! Rust bindings for the [OpenEXR](http://openexr.com) C++ library.
//!
//! OpenEXR is a bitmap image file format that can store high dynamic range
//! (HDR) images along with other arbitrary per-pixel data. It is used heavily
//! in the VFX and 3D animation industries.
//!
//! Although this wrapper's API differs a little from the C++ library, it tries
//! not to differ wildly.  Therefore the
//! [C++ OpenEXR documentation](https://github.com/openexr/openexr/tree/develop/OpenEXR/doc)
//! is still useful as an introduction and rough reference.  Moreover, the
//! file format itself is also documented there.
//!
//! # Overview
//!
//! There are three primary parts to this crate:
//!
//! * The various [input](input/index.html) and [output](output/index.html)
//!   types.  These are used for reading and writing OpenEXR files.  They
//!   utilize the Header and FrameBuffer(Mut) types, listed below.
//!
//! * [`Header`](struct.Header.html): this is used for querying and specifying
//!   the properties of an OpenEXR file (such as resolution, channels, etc.), for
//!   reading and writing respectively.
//!
//! * [`FrameBuffer`](frame_buffer/struct.FrameBuffer.html) and
//!   [`FrameBufferMut`](frame_buffer/struct.FrameBufferMut.html):
//!   these are intermediaries that tell the OpenEXR APIs how to interpret
//!   your in-memory image data.  Rather than passing your image data to the
//!   APIs directly, you construct a `FrameBuffer` that that points at and
//!   describes it, and then you pass that FrameBuffer.
//!
//! # Examples
//!
//! Writing a scanline floating point RGB file.
//!
//! ```no_run
//! # use openexr::{FrameBuffer, Header, ScanlineOutputFile, PixelType};
//! #
//! // Pixel data for a 256x256 floating point RGB image.
//! let pixel_data = vec![(0.82f32, 1.78f32, 0.21f32); 256 * 256];
//!
//! // Create a file to write to.  The `Header` determines the properties of the
//! // file, like resolution and what channels it has.
//! let mut file = std::fs::File::create("output_file.exr").unwrap();
//! let mut output_file = ScanlineOutputFile::new(
//!     &mut file,
//!     Header::new()
//!         .set_resolution(256, 256)
//!         .add_channel("R", PixelType::FLOAT)
//!         .add_channel("G", PixelType::FLOAT)
//!         .add_channel("B", PixelType::FLOAT)).unwrap();
//!
//! // Create a `FrameBuffer` that points at our pixel data and describes it as
//! // RGB data.
//! let mut fb = FrameBuffer::new(256, 256);
//! fb.insert_channels(&["R", "G", "B"], &pixel_data);
//!
//! // Write pixel data to the file.
//! output_file.write_pixels(&fb).unwrap();
//! ```
//!
//! Reading a floating point RGB file.
//!
//! ```no_run
//! # use openexr::{FrameBufferMut, InputFile};
//!
//! // Open the EXR file.
//! let mut file = std::fs::File::open("input_file.exr").unwrap();
//! let mut input_file = InputFile::new(&mut file).unwrap();
//!
//! // Get the image dimensions, so we know how large of a buffer to make.
//! let (width, height) = input_file.header().data_dimensions();
//!
//! // Buffer to read pixel data into.
//! let mut pixel_data = vec![(0.0f32, 0.0f32, 0.0f32); (width*height) as usize];
//!
//! // New scope because `FrameBuffer` mutably borrows `pixel_data`, so we need
//! // it to go out of scope before we can access our `pixel_data` again.
//! {
//!     // Create `FrameBufferMut` that points at our pixel data and describes
//!     // it as RGB data.
//!     let mut fb = FrameBufferMut::new(width, height);
//!     fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
//!
//!     // Read pixel data from the file.
//!     input_file.read_pixels(&mut fb).unwrap();
//! }
//! ```

#![warn(missing_docs)]
#![cfg_attr(feature = "unstable", feature(plugin))]
#![cfg_attr(feature = "unstable", plugin(clippy))]

extern crate half;
extern crate libc;
extern crate openexr_sys;

mod cexr_type_aliases;
mod stream_io;

pub mod error;
pub mod frame_buffer;
pub mod header;
pub mod input;
pub mod output;

pub use cexr_type_aliases::{Box2i, PixelType};
pub use error::{Error, Result};
pub use frame_buffer::{FrameBuffer, FrameBufferMut};
pub use header::{Envmap, Header};
pub use input::InputFile;
pub use output::ScanlineOutputFile;

/// Set the number of worker threads to use for compression/decompression.
///
/// This controls the maximum number of work threads that can be used to perform
/// compression,decompression while loading or writing a file. Note that the file I/O itself is
/// always performed on the calling thread. If this value is set to 0, multi-threaded is disabled
/// globally.
pub fn set_global_thread_count(thread_count: u32) -> Result<()> {
    if thread_count <= ::std::os::raw::c_int::max_value() as u32 {
        return Err(Error::Generic(String::from("The number of threads is too high")))
    }

    let error = unsafe {
        openexr_sys::CEXR_set_global_thread_count(thread_count as ::std::os::raw::c_int)
    };

	if error == 0 {
		Ok(())
	}
	else {
		Err(Error::Generic(String::from("Unable to set global thread count")))
	}
}
