extern crate openexr;

use std::slice;
use std::iter;
use std::path::Path;
use std::mem;

use openexr::{FrameBuffer, SliceDescription, ExrWriter, ExrWriterBuilder, Channel, PixelType};

fn main() {
    let mut pixel_data: Vec<(f32, f32, f32)> = iter::repeat((0.4, 0.2, 0.8)).take(256*256).collect();

    let mut wr = ExrWriterBuilder::new(Path::new("/tmp/test.exr"))
        .display_window((0,0), (255, 255))
        .data_window((0,0), (255, 255))
        .insert_channel("R", Channel::with_type(PixelType::F32))
        .insert_channel("G", Channel::with_type(PixelType::F32))
        .insert_channel("B", Channel::with_type(PixelType::F32))
        .open();

    let mut fb = {
        // Get the pixel data as a u8 slice.
        let p = pixel_data.as_mut_ptr();
        let l = pixel_data.len() * mem::size_of::<(f32, f32, f32)>();
        let pd = unsafe { slice::from_raw_parts_mut(p as *mut u8, l) };

        // Create the frame buffer
        let mut fb = FrameBuffer::new();
        fb.add_slice(pd,
            &[
                ("R", SliceDescription {
                    pixel_type: PixelType::F32,
                    start: 0,
                    stride: (4*3, 256*4*3),
                    subsampling: (1, 1),
                    tile_coords: (false, false),
                }, 0.0),
                ("G", SliceDescription {
                    pixel_type: PixelType::F32,
                    start: 4,
                    stride: (4*3, 256*4*3),
                    subsampling: (1, 1),
                    tile_coords: (false, false),
                }, 0.0),
                ("B", SliceDescription {
                    pixel_type: PixelType::F32,
                    start: 8,
                    stride: (4*3, 256*4*3),
                    subsampling: (1, 1),
                    tile_coords: (false, false),
                }, 0.0),
            ]
        );
        fb
    };

    wr.write_pixels(&mut fb, 256);
}
