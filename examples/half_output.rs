extern crate half;
extern crate openexr;

use std::env;
use std::fs::File;
use std::path::Path;

use half::f16;
use openexr::{FrameBuffer, Header, ScanlineOutputFile, PixelType};

fn main() {
    let mut pixel_data =
        vec![(f16::from_f32(0.82), f16::from_f32(1.78), f16::from_f32(0.21)); 256 * 256];

    let mut file = File::create(Path::new(&env::args_os().nth(1).expect("argument required")))
        .unwrap();

    let mut exr_file = ScanlineOutputFile::new(&mut file,
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
