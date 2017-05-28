use std::ffi::CStr;
use std::io::{Write, Seek};
use std::marker::PhantomData;
use std::ptr;

use openexr_sys::*;

use error::*;
use frame_buffer::FrameBuffer;
use Header;
use stream_io::{write_stream, seek_stream};

/// Writes scanline OpenEXR files.
///
/// This is the simplest kind of OpenEXR file.  Image data is stored in
/// scanline order with no special features like mipmaps or deep image data.
/// Unless you have a need for such special features, this is probably what
/// you want to use.
///
/// # Examples
///
/// Write a floating point RGB image to a file named "output_file.exr".
///
/// ```no_run
/// # use openexr::{ScanlineOutputFile, Header, FrameBuffer, PixelType};
/// #
/// // Create file with the desired resolution and channels.
/// let mut file = std::fs::File::create("output_file.exr").unwrap();
/// let mut output_file = ScanlineOutputFile::new(
///     &mut file,
///     Header::new()
///         .set_resolution(256, 256)
///         .add_channel("R", PixelType::FLOAT)
///         .add_channel("G", PixelType::FLOAT)
///         .add_channel("B", PixelType::FLOAT))
///     .unwrap();
///
/// // Create the image data and write it to the file.
/// let pixel_data = vec![(0.5f32, 1.0f32, 0.5f32); 256 * 256];
/// let mut fb = FrameBuffer::new(256, 256);
/// fb.insert_channels(&["R", "G", "B"], &pixel_data);
/// output_file.write_pixels(&fb);
/// ```
pub struct ScanlineOutputFile<'a> {
    handle: *mut CEXR_OutputFile,
    header_ref: Header,
    ostream: *mut CEXR_OStream,
    _phantom_1: PhantomData<CEXR_OutputFile>,
    _phantom_2: PhantomData<&'a mut ()>, // Represents the borrowed writer

    // NOTE: Because we don't know what type the writer might be, it's important
    // that this struct remains neither Sync nor Send.  Please don't implement
    // them!
}

impl<'a> ScanlineOutputFile<'a> {
    /// Creates a new `ScanlineOutputFile` from any `Write + Seek` type
    /// (typically a `std::fs::File`).
    ///
    /// Note: this seeks to byte 0 before writing.
    pub fn new<T: 'a>(writer: &'a mut T, header: &Header) -> Result<ScanlineOutputFile<'a>>
        where T: Write + Seek
    {
        let ostream_ptr = {
            let write_ptr = write_stream::<T>;
            let seekp_ptr = seek_stream::<T>;

            let mut error_out = ptr::null();
            let mut out = ptr::null_mut();
            let error = unsafe {
                CEXR_OStream_from_writer(writer as *mut T as *mut _,
                                         Some(write_ptr),
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
        let error = unsafe {
            // NOTE: we don't need to keep a copy of the header, because this
            // function makes a deep copy that is stored in the CEXR_OutputFile.
            CEXR_OutputFile_from_stream(ostream_ptr, header.handle, 1, &mut out, &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(ScanlineOutputFile {
                   handle: out,
                   header_ref: Header {
                       // NOTE: We're casting to *mut here to satisfy the
                       // field's type, but importantly we only return a
                       // const & of the Header so it retains const semantics.
                       handle: unsafe { CEXR_OutputFile_header(out) } as *mut CEXR_Header,
                       owned: false,
                       _phantom: PhantomData,
                   },
                   ostream: ostream_ptr,
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }

    /// Writes image data from the given FrameBuffer.
    pub fn write_pixels(&mut self, framebuffer: &FrameBuffer) -> Result<()> {
        framebuffer.validate_header_for_output(self.header())?;

        let mut error_out = ptr::null();

        let error = unsafe {
            CEXR_OutputFile_set_framebuffer(self.handle, framebuffer.handle(), &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            return Err(Error::Generic(msg.to_string_lossy().into_owned()));
        }

        let error = unsafe {
            CEXR_OutputFile_write_pixels(self.handle,
                                         framebuffer.dimensions().1 as i32,
                                         &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(())
        }
    }

    /// Access to the file's header.
    pub fn header(&self) -> &Header {
        &self.header_ref
    }
}

impl<'a> Drop for ScanlineOutputFile<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_OutputFile_delete(self.handle) };
        unsafe { CEXR_OStream_delete(self.ostream) };
    }
}
