extern crate openexr;

use std::env;
use std::fs::File;
use std::path::Path;

use openexr::{FrameBuffer, InputFile, PixelType};

fn main() {
    // Open the EXR file and get its dimensions.
    let mut file = File::open(Path::new(&env::args_os().nth(1).expect("argument required")))
        .unwrap();
    let exr_file = InputFile::new(&mut file).unwrap();
    let window = exr_file.header().data_window();
    let width = window.max.x - window.min.x + 1;
    let height = window.max.y - window.min.y + 1;

    // Make sure the channels we want exist in the file
    for channel_name in ["R", "G", "B"].iter() {
        let channel = exr_file
            .header()
            .get_channel(channel_name)
            .expect(&format!("Didn't find channel {}.", channel_name));
        assert!(channel.pixel_type == PixelType::FLOAT);
    }

    // Create our pixel data buffer and load the data from the file
    let mut pixel_data: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); (width*height) as usize];

    {
        let mut fb = {
            // Create the frame buffer
            let mut fb = FrameBuffer::new(width as usize, height as usize);
            fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
            fb
        };

        exr_file.read_pixels(&mut fb).unwrap();
    }

    // Verify the data is what we expect
    for pixel in pixel_data {
        assert_eq!(pixel, (0.82, 1.78, 0.21));
    }
}
