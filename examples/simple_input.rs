extern crate openexr;

use std::path::Path;

use openexr::{FrameBuffer, ExrReader};

fn main() {
    let mut pixel_data: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); 256*256];

    let mut re = ExrReader::new(Path::new("/tmp/test.exr")).unwrap();

    {
        let mut fb = {
            // Create the frame buffer
            let mut fb = FrameBuffer::new(256, 256);
            fb.add_structured_slice(
                &mut pixel_data,
                &[("R", 0.0), ("G", 0.0), ("B", 0.0)]
            );
            fb
        };

        re.read_pixels(&mut fb).unwrap();
    }

    for pixel in pixel_data {
        if pixel != (0.82, 1.78, 0.21) {
            panic!("unexpected pixel value: ({}, {}, {})", pixel.0, pixel.1, pixel.2);
        }
    }
}
