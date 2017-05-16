extern crate openexr;

use std::slice;
use std::iter;
use std::path::Path;
use std::mem;

use openexr::{FrameBuffer, SliceDescription, ExrWriterBuilder, Channel, PixelType};

fn main() {
    let mut pixel_data: Vec<(f32, f32, f32)> =
        iter::repeat((0.82, 1.78, 0.21)).take(256 * 256).collect();

    let mut wr = ExrWriterBuilder::new(Path::new("/tmp/test.exr"))
        .display_window((0, 0), (255, 255))
        .data_window((0, 0), (255, 255))
        .insert_channel("R", Channel::with_type(PixelType::F32))
        .insert_channel("G", Channel::with_type(PixelType::F32))
        .insert_channel("B", Channel::with_type(PixelType::F32))
        .open();

    let mut fb = {
        // Create the frame buffer
        let mut fb = FrameBuffer::new(256, 256);
        fb.add_structured_slice(&mut pixel_data, &[("R", 0.0), ("G", 0.0), ("B", 0.0)]);
        fb
    };

    wr.write_pixels(&mut fb);
}
