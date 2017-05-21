extern crate half;
extern crate openexr;

use std::iter;
use std::path::Path;

use half::f16;
use openexr::{FrameBuffer, Header, ScanlineOutputFile, PixelType};

fn main() {
    let mut pixel_data: Vec<(f16, f16, f16)> =
        iter::repeat((f16::from_f32(0.82), f16::from_f32(1.78), f16::from_f32(0.21)))
            .take(256 * 256)
            .collect();

    let mut exr_file = ScanlineOutputFile::new(Path::new("/tmp/test.exr"),
                                               &Header::new()
                                                    .set_resolution(256, 256)
                                                    .add_channel("R", PixelType::HALF)
                                                    .add_channel("G", PixelType::HALF)
                                                    .add_channel("B", PixelType::HALF))
            .unwrap();

    let mut fb = {
        // Create the frame buffer
        let mut fb = FrameBuffer::new(256, 256);
        fb.insert_pixels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        fb
    };

    exr_file.write_pixels(&mut fb).unwrap();
}
