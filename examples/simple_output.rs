extern crate openexr;

use std::iter;
use std::path::Path;

use openexr::{FrameBuffer, OutputFile, PixelType, Compression};

fn main() {
    let mut pixel_data: Vec<(f32, f32, f32)> =
        iter::repeat((0.82, 1.78, 0.21)).take(256 * 256).collect();

    let mut exr_file = OutputFile::new()
        .resolution(256, 256)
        .channel("R", PixelType::FLOAT)
        .channel("G", PixelType::FLOAT)
        .channel("B", PixelType::FLOAT)
        .open(Path::new("/tmp/test.exr"))
        .unwrap();

    let mut fb = {
        // Create the frame buffer
        let mut fb = FrameBuffer::new(256, 256);
        fb.insert_pixels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        fb
    };

    exr_file.write_pixels(&mut fb).unwrap();
}
