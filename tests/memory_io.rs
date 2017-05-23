extern crate openexr;

use std::env;
use std::io::Cursor;
use std::path::Path;

use openexr::{FrameBuffer, Header, ScanlineOutputFile, InputFile, PixelType};

fn main() {
    let mut in_memory_buffer = Cursor::new(Vec::<u8>::new());

    {
        let mut pixel_data = vec![(0.82f32, 1.78f32, 0.21f32); 256 * 256];

        let mut exr_file =
            ScanlineOutputFile::from_writer(&mut in_memory_buffer,
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

    {
        let mut pixel_data = vec![(0.0f32, 0.0f32, 0.0f32); 256 * 256];

        let exr_file = InputFile::from_slice(in_memory_buffer.get_ref()).unwrap();
        let window = exr_file.header().data_window();
        let width = window.max.x - window.min.x + 1;
        let height = window.max.y - window.min.y + 1;

        // Make sure the image properties are the same.
        assert!(width == 256);
        assert!(height == 256);
        for channel_name in ["R", "G", "B"].iter() {
            let channel = exr_file
                .header()
                .get_channel(channel_name)
                .expect(&format!("Didn't find channel {}.", channel_name));
            assert!(channel.pixel_type == PixelType::FLOAT);
        }

        // Read in the pixel data.
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
}
