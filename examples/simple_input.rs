extern crate openexr;

use std::env;
use std::path::Path;

use openexr::{FrameBuffer, InputFile, PixelType};

fn main() {
    // Open the EXR file and get its dimensions.
    let exr_file = InputFile::new(Path::new(&env::args_os().nth(1).expect("argument required"))).unwrap();
    let window = exr_file.header().data_window();
    let width = window.max.x - window.min.x + 1;
    let height = window.max.y - window.min.y + 1;

    // Make sure the channels we want exist in the file
    assert!(exr_file
                .header()
                .get_channel("R")
                .expect("Didn't find channel 'R'.")
                .pixel_type == PixelType::FLOAT);
    assert!(exr_file
                .header()
                .get_channel("G")
                .expect("Didn't find channel 'G'.")
                .pixel_type == PixelType::FLOAT);
    assert!(exr_file
                .header()
                .get_channel("B")
                .expect("Didn't find channel 'B'.")
                .pixel_type == PixelType::FLOAT);

    // Create our pixel data buffer and load the data from the file
    let mut pixel_data: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); (width*height) as usize];

    {
        let mut fb = {
            // Create the frame buffer
            let mut fb = FrameBuffer::new(width as usize, height as usize);
            fb.insert_pixels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
            fb
        };

        exr_file.read_pixels(&mut fb).unwrap();
    }

    // Verify the data is what we expect
    for pixel in pixel_data {
        assert_eq!(pixel, (0.82, 1.78, 0.21));
    }
}
