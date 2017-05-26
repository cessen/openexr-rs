extern crate openexr;

use std::env;
use std::fs::File;
use std::iter;
use std::path::Path;

use openexr::{FrameBuffer, Header, ScanlineOutputFile, PixelType};

fn main() {
    let mut pixel_data: Vec<(f32, f32, f32)> =
        iter::repeat((0.82, 1.78, 0.21)).take(256 * 256).collect();

    let mut file = File::create(Path::new(&env::args_os().nth(1).expect("argument required")))
        .unwrap();

    let mut exr_file = ScanlineOutputFile::new(&mut file,
                                               &Header::new()
                                                    .set_resolution(256, 256)
                                                    .add_channel("R", PixelType::FLOAT)
                                                    .add_channel("G", PixelType::FLOAT)
                                                    .add_channel("B", PixelType::FLOAT))
            .unwrap();

    let mut fb = {
        // Create the frame buffer
        let mut fb = FrameBuffer::new(256, 256);
        fb.insert_pixels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        fb
    };

    exr_file.write_pixels(&mut fb).unwrap();
}
