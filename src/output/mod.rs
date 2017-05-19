mod scanline_writer;

use std::ffi::CString;
use std::marker::PhantomData;
use std::path::Path;

use openexr_sys::*;

use cexr_type_aliases::*;
use error::*;

pub use self::scanline_writer::ScanlineWriter;


pub struct OutputFile {
    header_handle: *mut CEXR_Header,
    _phantom_1: PhantomData<CEXR_OutputFile>,
    _phantom_2: PhantomData<CEXR_Header>,
}

impl OutputFile {
    pub fn new() -> OutputFile {
        // Create header
        let header = {
            let display_window = Box2i {
                min: CEXR_V2i { x: 0, y: 0 },
                max: CEXR_V2i { x: 1, y: 1 },
            };
            let data_window = display_window;
            let pixel_aspect_ratio = 1.0;
            let screen_window_center = CEXR_V2f { x: 0.0, y: 0.0 };
            let screen_window_width = 1.0;
            let line_order = LineOrder::INCREASING_Y;
            let compression = Compression::PIZ_COMPRESSION;
            let header = unsafe {
                CEXR_Header_new(&display_window,
                                &data_window,
                                pixel_aspect_ratio,
                                &screen_window_center,
                                screen_window_width,
                                line_order,
                                compression)
            };
            header
        };

        OutputFile {
            header_handle: header,
            _phantom_1: PhantomData,
            _phantom_2: PhantomData,
        }
    }

    /// Sets the resolution.
    ///
    /// This is really just a shortcut for setting both the display window
    /// and data window to `(0, 0), (width-1, height-1)`.
    pub fn resolution(self, width: u32, height: u32) -> OutputFile {
        let window = Box2i {
            min: CEXR_V2i { x: 0, y: 0 },
            max: CEXR_V2i {
                x: width as i32 - 1,
                y: height as i32 - 1,
            },
        };
        unsafe {
            CEXR_Header_set_display_window(self.header_handle, window);
        }
        unsafe {
            CEXR_Header_set_data_window(self.header_handle, window);
        }
        self
    }

    /// Sets the display window.
    pub fn display_window(self, window: Box2i) -> OutputFile {
        unsafe {
            CEXR_Header_set_display_window(self.header_handle, window);
        }
        self
    }

    /// Sets the data window.
    pub fn data_window(self, window: Box2i) -> OutputFile {
        unsafe {
            CEXR_Header_set_data_window(self.header_handle, window);
        }
        self
    }

    /// Sets the pixel aspect ratio.
    pub fn pixel_aspect_ratio(self, aspect_ratio: f32) -> OutputFile {
        unsafe {
            CEXR_Header_set_pixel_aspect_ratio(self.header_handle, aspect_ratio);
        }
        self
    }

    /// Sets the screen window center.
    pub fn screen_window_center(self, center: (f32, f32)) -> OutputFile {
        unsafe {
            CEXR_Header_set_screen_window_center(self.header_handle,
                                                 CEXR_V2f {
                                                     x: center.0,
                                                     y: center.1,
                                                 });
        }
        self
    }

    /// Sets the screen window width.
    pub fn screen_window_width(self, width: f32) -> OutputFile {
        unsafe {
            CEXR_Header_set_screen_window_width(self.header_handle, width);
        }
        self
    }

    /// Sets the line order.
    pub fn line_order(self, line_order: LineOrder) -> OutputFile {
        unsafe {
            CEXR_Header_set_line_order(self.header_handle, line_order);
        }
        self
    }

    /// Sets the compression mode.
    pub fn compression(self, compression: Compression) -> OutputFile {
        unsafe {
            CEXR_Header_set_compression(self.header_handle, compression);
        }
        self
    }

    /// Adds a channel.
    ///
    /// This is a simplified version of `channel_detailed()`, using some sane
    /// defaults for the details.  Specifially: sampling is set to (1, 1)
    /// and p_linear is set to true.
    pub fn channel(self, name: &str, pixel_type: PixelType) -> OutputFile {
        self.channel_detailed(name,
                              Channel {
                                  pixel_type: pixel_type,
                                  x_sampling: 1,
                                  y_sampling: 1,
                                  p_linear: true,
                              })
    }

    /// Adds a channel, specifying full details.
    pub fn channel_detailed(self, name: &str, channel: Channel) -> OutputFile {
        let cname = CString::new(name.as_bytes()).unwrap();
        unsafe { CEXR_Header_insert_channel(self.header_handle, cname.as_ptr(), channel) };
        self
    }

    /// Opens the file as a simple scanline OpenEXR file.
    pub fn open(self, path: &Path) -> Result<ScanlineWriter> {
        // Create file
        let c_path = CString::new(path.to_str()
                                      .expect("non-unicode path handling is unimplemented")
                                      .as_bytes())
                .unwrap();
        ScanlineWriter::new(c_path.as_ptr(), self.header_handle)
    }
}

impl Drop for OutputFile {
    fn drop(&mut self) {
        unsafe { CEXR_Header_delete(self.header_handle) };
    }
}
