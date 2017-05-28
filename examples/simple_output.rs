extern crate openexr;

use std::env;
use std::fs::File;
use std::path::Path;

use openexr::{FrameBuffer, Header, ScanlineOutputFile, PixelType};

fn main() {
    let pixel_data = vec![(0.82f32, 1.78f32, 0.21f32); 256 * 256];

    let mut file = File::create(Path::new(&env::args_os().nth(1).expect("argument required")))
        .unwrap();

    let mut exr_file = ScanlineOutputFile::new(&mut file,
                                               &Header::new()
                                                    .set_resolution(256, 256)
                                                    .add_channel("R", PixelType::FLOAT)
                                                    .add_channel("G", PixelType::FLOAT)
                                                    .add_channel("B", PixelType::FLOAT))
            .unwrap();

    let fb = {
        // Create the frame buffer
        let mut fb = FrameBuffer::new(256, 256);
        fb.insert_channels(&["R", "G", "B"], &pixel_data);
        fb
    };

    exr_file.write_pixels(&fb).unwrap();
}
