extern crate libc;
extern crate openexr_sys;

use std::path::Path;
use std::collections::HashMap;
use std::ffi::CString;

use libc::{c_char, c_int, c_float};

use openexr_sys as cexr;

pub use openexr_sys::CEXR_PixelType as PixelType;
pub use openexr_sys::CEXR_CompressionMethod as CompressionMethod;
pub use openexr_sys::CEXR_LineOrder as LineOrder;


// ------------------------------------------------------------------------------

pub struct Box2i {
    pub min: (i32, i32),
    pub max: (i32, i32),
}


// ------------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub struct Channel {
    pub pixel_type: PixelType,
    pub subsampling: (u32, u32), /* If set to 1, every pixel, if set to 2, every
                                  * other pixel, if 3, every third, etc. */
    pub p_linear: bool, /* Hint to lossy compression methods that indicates whether
                         * human perception of the quantity represented by this channel
                         * is closer to linear or closer to logarithmic. */
}

impl Channel {
    pub fn with_type(pixel_type: PixelType) -> Channel {
        Channel {
            pixel_type: pixel_type,
            subsampling: (1, 1),
            p_linear: true,
        }
    }
}


// ------------------------------------------------------------------------------

#[derive(Copy, Clone)]
pub struct SliceDescription {
    pub pixel_type: PixelType,
    pub start: usize,
    pub stride: (usize, usize),
    pub subsampling: (usize, usize),
    pub tile_coords: (bool, bool),
}

pub struct FrameBuffer<'a> {
    channels: HashMap<&'a str, (usize, SliceDescription, f64)>,
    buffers: Vec<&'a mut [u8]>,
}

impl<'a> FrameBuffer<'a> {
    pub fn new() -> FrameBuffer<'a> {
        FrameBuffer {
            channels: HashMap::new(),
            buffers: Vec::new(),
        }
    }

    pub fn add_slice(&mut self,
                     data: &'a mut [u8],
                     descriptions: &[(&'a str, SliceDescription, f64)]) {
        // Make sure we're not creating any duplicate channels.
        // We check this ahead of time instead of in the same loop as inserting the channels
        // so that the FrameBuffer state remains valid.
        for &(name, _, _) in descriptions {
            if self.channels.contains_key(name) {
                // TODO: return an Err instead of a panicing
                panic!("Cannot have two of the same channel name in a FrameBuffer.");
            }
        }

        // Insert channels
        for &(name, desc, default) in descriptions {
            self.channels.insert(name, (self.buffers.len(), desc, default));
        }

        // Add buffer
        self.buffers.push(data);
    }
}


// ------------------------------------------------------------------------------

pub struct ExrWriterBuilder<'a> {
    path: &'a Path,
    display_window: Box2i,
    data_window: Box2i,
    pixel_aspect_ratio: f32,
    screen_window_center: (f32, f32),
    screen_window_width: f32,
    line_order: LineOrder,
    compression_method: CompressionMethod,
    channels: HashMap<String, Channel>,
}

impl<'a> ExrWriterBuilder<'a> {
    pub fn new(path: &'a Path) -> ExrWriterBuilder<'a> {
        ExrWriterBuilder {
            path: path,
            display_window: Box2i {
                min: (0, 0),
                max: (0, 0),
            },
            data_window: Box2i {
                min: (0, 0),
                max: (0, 0),
            },
            pixel_aspect_ratio: 1.0,
            screen_window_center: (0.0, 0.0),
            screen_window_width: 1.0,
            line_order: LineOrder::IncreasingY,
            compression_method: CompressionMethod::ZIP,
            channels: HashMap::new(),
        }
    }

    pub fn display_window(mut self, min: (i32, i32), max: (i32, i32)) -> ExrWriterBuilder<'a> {
        self.display_window = Box2i {
            min: min,
            max: max,
        };
        self
    }

    pub fn data_window(mut self, min: (i32, i32), max: (i32, i32)) -> ExrWriterBuilder<'a> {
        self.data_window = Box2i {
            min: min,
            max: max,
        };
        self
    }

    pub fn pixel_aspect_ratio(mut self, par: f32) -> ExrWriterBuilder<'a> {
        self.pixel_aspect_ratio = par;
        self
    }

    pub fn screen_window_center(mut self, swc: (f32, f32)) -> ExrWriterBuilder<'a> {
        self.screen_window_center = swc;
        self
    }

    pub fn screen_window_width(mut self, sww: f32) -> ExrWriterBuilder<'a> {
        self.screen_window_width = sww;
        self
    }

    pub fn line_order(mut self, lo: LineOrder) -> ExrWriterBuilder<'a> {
        self.line_order = lo;
        self
    }

    pub fn compression_method(mut self, cm: CompressionMethod) -> ExrWriterBuilder<'a> {
        self.compression_method = cm;
        self
    }

    pub fn insert_channel(mut self, name: &str, channel: Channel) -> ExrWriterBuilder<'a> {
        self.channels.insert(name.to_string(), channel);
        self
    }

    pub fn open(self) -> ExrWriter {
        // Build the header
        let header = {
            let mut header = unsafe {
                cexr::CEXR_Header_new(self.display_window.min.0 as c_int,
                                      self.display_window.min.1 as c_int,
                                      self.display_window.max.0 as c_int,
                                      self.display_window.max.1 as c_int,
                                      self.data_window.min.0 as c_int,
                                      self.data_window.min.1 as c_int,
                                      self.data_window.max.0 as c_int,
                                      self.data_window.max.1 as c_int,
                                      self.pixel_aspect_ratio as c_float,
                                      self.screen_window_center.0,
                                      self.screen_window_center.1,
                                      self.screen_window_width,
                                      self.line_order,
                                      self.compression_method)
            };
            for (name, channel) in &self.channels {
                let n = CString::new(name.as_bytes()).unwrap();
                let c = cexr::CEXR_Channel {
                    pixel_type: channel.pixel_type,
                    x_sampling: channel.subsampling.0 as c_int,
                    y_sampling: channel.subsampling.1 as c_int,
                    p_linear: if channel.p_linear {
                        1 as c_int
                    } else {
                        0 as c_int
                    },
                };
                unsafe { cexr::CEXR_Header_insert_channel(&mut header, n.as_ptr(), c) };
            }
            header
        };



        ExrWriter {
            handle: unsafe {
                cexr::CEXR_OutputFile_new(CString::new(self.path.to_str().unwrap().as_bytes())
                                              .unwrap()
                                              .as_ptr(),
                                          &header,
                                          1)
            },
        }
    }
}


// ------------------------------------------------------------------------------
pub struct ExrWriter {
    handle: cexr::CEXR_OutputFile,
}

impl Drop for ExrWriter {
    fn drop(&mut self) {
        unsafe { cexr::CEXR_OutputFile_delete(&mut self.handle) };
    }
}

impl ExrWriter {
    pub fn write_pixels(&mut self, frame_buffer: &mut FrameBuffer, num_scan_lines: usize) {
        // Build the frame buffer.
        let mut cexr_fb = {
            let mut cexr_fb = unsafe { cexr::CEXR_FrameBuffer_new() };
            for (&name, &(buf_index, desc, default)) in frame_buffer.channels.iter() {
                let n = CString::new(name.as_bytes()).unwrap();
                let buf_ptr = unsafe {
                    frame_buffer.buffers[buf_index].as_mut_ptr().offset(desc.start as isize)
                } as *mut c_char;
                unsafe {
                    cexr::CEXR_FrameBuffer_insert_slice(&mut cexr_fb,
                                                        n.as_ptr(),
                                                        desc.pixel_type,
                                                        buf_ptr,
                                                        desc.stride.0,
                                                        desc.stride.1,
                                                        desc.subsampling.0 as i32,
                                                        desc.subsampling.1 as i32,
                                                        default,
                                                        desc.tile_coords.0 as i32,
                                                        desc.tile_coords.1 as i32)
                };
            }
            cexr_fb
        };

        // Set the frame buffer.
        unsafe { cexr::CEXR_OutputFile_set_frame_buffer(&mut self.handle, &mut cexr_fb) };

        // Write the pixel data.
        unsafe { cexr::CEXR_OutputFile_write_pixels(&mut self.handle, num_scan_lines as i32) };

        // Destroy the framebuffer
        unsafe { cexr::CEXR_FrameBuffer_delete(&mut cexr_fb) };
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!("woo hoo!");
    }
}
