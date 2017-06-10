//! Input file types.

use std::cmp::min;
use std::ffi::CStr;
use std::io::{Read, Seek};
use std::marker::PhantomData;
use std::ptr;

use libc::c_char;

use openexr_sys::*;

use error::*;
use frame_buffer::FrameBufferMut;
use Header;
use stream_io::{read_stream, seek_stream};

/// Reads any kind of OpenEXR file.
///
/// `InputFile` is a bit unique in that it doesn't care what kind of OpenEXR
/// file is being read.  Regardless of the type being read, it presents an API
/// as if it were a basic scanline OpenEXR file.
///
/// Special features like tiles, mipmaps, and deep image data will not be
/// available even if they are present in the file.  To gain access to those
/// features you need to use the other input file types (not yet implemented,
/// sorry!).
///
/// # Examples
///
/// Load image data from a floating point RGB image file named "input_file.exr".
///
/// ```no_run
/// # use openexr::{InputFile, FrameBufferMut};
/// #
/// // Open file and get its resolution.
/// let mut file = std::fs::File::open("input_file.exr").unwrap();
/// let mut input_file = InputFile::new(&mut file).unwrap();
/// let (width, height) = input_file.header().data_dimensions();
///
/// // Allocate a buffer for the image data and read it in.
/// let mut pixel_data: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 0.0]; (width*height) as usize];
/// let mut fb = FrameBufferMut::new(width, height);
/// fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0), ("A", 0.0)], &mut pixel_data);
/// input_file.read_pixels(&mut fb).unwrap();
/// ```
#[allow(dead_code)]
pub struct InputFile<'a> {
    handle: *mut CEXR_InputFile,
    header_ref: Header,
    istream: *mut CEXR_IStream,
    _phantom_1: PhantomData<CEXR_InputFile>,
    _phantom_2: PhantomData<&'a mut ()>, // Represents the borrowed reader

    // NOTE: Because we don't know what type the reader might be, it's important
    // that this struct remains neither Sync nor Send.  Please don't implement
    // them!
}

impl<'a> InputFile<'a> {
    /// Creates a new `InputFile` from any `Read + Seek` type (typically a
    /// `std::fs::File`).
    ///
    /// Note: this seeks to byte 0 before reading.
    pub fn new<T: 'a>(reader: &mut T) -> Result<InputFile>
        where T: Read + Seek
    {
        let istream_ptr = {
            let read_ptr = read_stream::<T>;
            let seekp_ptr = seek_stream::<T>;

            let mut error_out = ptr::null();
            let mut out = ptr::null_mut();
            let error = unsafe {
                CEXR_IStream_from_reader(reader as *mut T as *mut _,
                                         Some(read_ptr),
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
        let error = unsafe { CEXR_InputFile_from_stream(istream_ptr, 1, &mut out, &mut error_out) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(InputFile {
                   handle: out,
                   header_ref: Header {
                       // NOTE: We're casting to *mut here to satisfy the
                       // field's type, but importantly we only return a
                       // const & of the Header so it retains const semantics.
                       handle: unsafe { CEXR_InputFile_header(out) } as *mut CEXR_Header,
                       owned: false,
                       _phantom: PhantomData,
                   },
                   istream: istream_ptr,
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }

    /// Creates a new `InputFile` from a slice of bytes, reading from memory.
    ///
    /// Note: although you can do essentially the same thing by passing a
    /// `std::io::Cursor<&[u8]>` to `new()`, using this method is more
    /// efficient because it allows the underlying APIs to avoid reading data
    /// into intermediate buffers.
    pub fn from_slice(slice: &[u8]) -> Result<InputFile> {
        let istream_ptr = unsafe {
            CEXR_IStream_from_memory(b"in-memory data\0".as_ptr() as *const c_char,
                                     slice.as_ptr() as *mut u8 as *mut c_char,
                                     slice.len())
        };

        let mut error_out = ptr::null();
        let mut out = ptr::null_mut();
        let error = unsafe { CEXR_InputFile_from_stream(istream_ptr, 1, &mut out, &mut error_out) };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(InputFile {
                   handle: out,
                   header_ref: Header {
                       // NOTE: We're casting to *mut here to satisfy the
                       // field's type, but importantly we only return a
                       // const & of the Header so it retains const semantics.
                       handle: unsafe { CEXR_InputFile_header(out) } as *mut CEXR_Header,
                       owned: false,
                       _phantom: PhantomData,
                   },
                   istream: istream_ptr,
                   _phantom_1: PhantomData,
                   _phantom_2: PhantomData,
               })
        }
    }


    /// Reads the entire image into `framebuffer` at once.
    ///
    /// Any channels in `framebuffer` that are not present in the file will be
    /// filled with their default fill value.
    ///
    /// # Errors
    ///
    /// This function expects `framebuffer` to have the same resolution as the
    /// file, and for any same-named channels to have matching types and
    /// subsampling.
    ///
    /// It will also return an error if there is an I/O error.
    pub fn read_pixels(&mut self, framebuffer: &mut FrameBufferMut) -> Result<()> {
        // ^^^ NOTE: it's not obvious, but this does indeed need to take self as
        // &mut to be safe.  Even though it is not conceptually modifying the
        // thing (typically a file) that it's reading from, it still has a
        // cursor getting incremented etc. during reads, so the reference needs
        // to be unique to avoid unsafe aliasing.

        // Validation
        if self.header().data_dimensions() != framebuffer.dimensions() {
            return Err(Error::Generic(format!("framebuffer size {}x{} does not match \
                                              image dimensions {}x{}",
                                              framebuffer.dimensions().0,
                                              framebuffer.dimensions().1,
                                              self.header().data_dimensions().0,
                                              self.header().data_dimensions().1)));
        }

        self.header().validate_framebuffer_for_input(framebuffer)?;

        // Set up the framebuffer with the image
        let mut error_out = ptr::null();

        let error = unsafe {
            CEXR_InputFile_set_framebuffer(self.handle, framebuffer.handle_mut(), &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            return Err(Error::Generic(msg.to_string_lossy().into_owned()));
        }

        // Read the image data
        let error = unsafe {
            CEXR_InputFile_read_pixels(self.handle,
                                       self.header().data_window().min.y,
                                       self.header().data_window().max.y,
                                       &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok(())
        }
    }

    /// Reads a contiguous chunk of scanlines into `framebuffer`.
    ///
    /// `framebuffer` may have a different vertical resolution than the image,
    /// but must have the same horizontal resolution.  Scanlines are read from
    /// the image starting at `starting_scanline` and are written to
    /// `framebuffer` until either its or the image's last scanline is reached.
    ///
    /// For example, to read the last 50 scanlines of a 200-pixel-tall image,
    /// you would pass a 50-pixel-tall FrameBufferMut and a starting scanline of
    /// 150.
    ///
    /// Any channels in `framebuffer` that are not present in the file will be
    /// filled with their default fill value.
    ///
    /// On success returns the number of scanlines read.
    ///
    /// # Errors
    ///
    /// This function expects `framebuffer` to have the same _horizontal_
    /// resolution as the file, and for any same-named channels to have
    /// matching types and subsampling.
    ///
    /// It will also return an error if there is an I/O error.
    pub fn read_pixels_partial(&mut self,
                               starting_scanline: u32,
                               framebuffer: &mut FrameBufferMut)
                               -> Result<(u32)> {
        // ^^^ NOTE: it's not obvious, but this does indeed need to take self as
        // &mut to be safe.  Even though it is not conceptually modifying the
        // thing (typically a file) that it's reading from, it still has a
        // cursor getting incremented etc. during reads, so the reference needs
        // to be unique to avoid unsafe aliasing.

        // Validation
        assert!(starting_scanline < self.header().data_dimensions().1,
                "Cannot start reading past last scanline.");

        if self.header().data_dimensions().0 != framebuffer.dimensions().0 {
            return Err(Error::Generic(format!("framebuffer width {} does not match\
                                              image width {}",
                                              framebuffer.dimensions().0,
                                              self.header().data_dimensions().0)));
        }

        self.header().validate_framebuffer_for_input(framebuffer)?;

        // Set up the framebuffer with the image
        let scanline_read_count = min(self.header().data_dimensions().1 - starting_scanline,
                                      framebuffer.dimensions().1);
        let start_scanline = self.header().data_window().min.y + starting_scanline as i32;
        let end_scanline = self.header().data_window().min.y +
                           (starting_scanline + scanline_read_count) as i32 -
                           1;

        let mut error_out = ptr::null();

        let error = unsafe {
            let offset_fb = CEXR_FrameBuffer_copy_and_offset_scanlines(framebuffer.handle_mut(),
                                                                       starting_scanline);
            let err = CEXR_InputFile_set_framebuffer(self.handle, offset_fb, &mut error_out);
            CEXR_FrameBuffer_delete(offset_fb);
            err
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            return Err(Error::Generic(msg.to_string_lossy().into_owned()));
        }

        // Read the image data
        let error = unsafe {
            CEXR_InputFile_read_pixels(self.handle, start_scanline, end_scanline, &mut error_out)
        };
        if error != 0 {
            let msg = unsafe { CStr::from_ptr(error_out) };
            Err(Error::Generic(msg.to_string_lossy().into_owned()))
        } else {
            Ok((scanline_read_count))
        }
    }

    /// Access to the file's header.
    pub fn header(&self) -> &Header {
        &self.header_ref
    }
}

impl<'a> Drop for InputFile<'a> {
    fn drop(&mut self) {
        unsafe { CEXR_InputFile_delete(self.handle) };
        unsafe { CEXR_IStream_delete(self.istream) };
    }
}
