extern crate openexr;

use std::io::Cursor;

use openexr::{FrameBuffer, FrameBufferMut, Header, ScanlineOutputFile, InputFile, PixelType};

#[test]
fn incremental_io() {
    // Target memory for writing
    let mut in_memory_buffer = Cursor::new(Vec::<u8>::new());

    // Write file to memory
    {
        let mut exr_file = ScanlineOutputFile::new(&mut in_memory_buffer,
                                                   &Header::new()
                                                        .set_resolution(256, 256)
                                                        .add_channel("R", PixelType::FLOAT)
                                                        .add_channel("G", PixelType::FLOAT)
                                                        .add_channel("B", PixelType::FLOAT))
                .unwrap();

        // Write incrementally with four calls, using different colors
        // for each call.
        let mut pixel_data = vec![(1.0f32, 0.0f32, 0.0f32); 256 * 64];

        exr_file
            .write_pixels_incremental(FrameBuffer::new(256, 64)
                                          .insert_channels(&["R", "G", "B"], &pixel_data))
            .unwrap();

        for pixel in pixel_data.iter_mut() {
            *pixel = (0.0, 1.0, 0.0);
        }

        exr_file
            .write_pixels_incremental(FrameBuffer::new(256, 64)
                                          .insert_channels(&["R", "G", "B"], &pixel_data))
            .unwrap();

        for pixel in pixel_data.iter_mut() {
            *pixel = (0.0, 0.0, 1.0);
        }

        exr_file
            .write_pixels_incremental(FrameBuffer::new(256, 64)
                                          .insert_channels(&["R", "G", "B"], &pixel_data))
            .unwrap();

        for pixel in pixel_data.iter_mut() {
            *pixel = (1.0, 1.0, 1.0);
        }

        exr_file
            .write_pixels_incremental(FrameBuffer::new(256, 64)
                                          .insert_channels(&["R", "G", "B"], &pixel_data))
            .unwrap();
    }

    // // Read file from memory, and verify its contents
    // {
    //     let mut pixel_data = vec![(0.0f32, 0.0f32, 0.0f32); 256 * 256];

    //     let mut exr_file = InputFile::from_slice(in_memory_buffer.get_ref()).unwrap();
    //     let (width, height) = exr_file.header().data_dimensions();

    //     // Make sure the image properties are the same.
    //     assert!(width == 256);
    //     assert!(height == 256);
    //     for channel_name in ["R", "G", "B"].iter() {
    //         let channel = exr_file
    //             .header()
    //             .get_channel(channel_name)
    //             .expect(&format!("Didn't find channel {}.", channel_name));
    //         assert!(channel.pixel_type == PixelType::FLOAT);
    //     }

    //     // Read in the pixel data.
    //     {
    //         let mut fb = FrameBufferMut::new(width, height);
    //         fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);

    //         exr_file.read_pixels(&mut fb).unwrap();
    //     }

    //     // Verify the data is what we expect
    //     for pixel in pixel_data {
    //         assert_eq!(pixel, (0.82, 1.78, 0.21));
    //     }
    // }
}
