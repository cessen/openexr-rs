extern crate openexr;

use std::io::Cursor;

use openexr::{FrameBuffer, FrameBufferMut, Header, InputFile, PixelType, ScanlineOutputFile};

#[test]
fn incremental_io() {
    // Target memory for writing
    let mut in_memory_buffer = Cursor::new(Vec::<u8>::new());

    // Write file to memory
    {
        let mut exr_file = ScanlineOutputFile::new(
            &mut in_memory_buffer,
            &Header::new()
                .set_resolution(256, 256)
                .add_channel("R", PixelType::FLOAT)
                .add_channel("G", PixelType::FLOAT)
                .add_channel("B", PixelType::FLOAT),
        ).unwrap();

        // Write incrementally with four calls, using different colors
        // for each call.
        let mut pixel_data = vec![(1.0f32, 0.0f32, 0.0f32); 256 * 64];

        exr_file
            .write_pixels_incremental(
                FrameBuffer::new(256, 64).insert_channels(&["R", "G", "B"], &pixel_data),
            ).unwrap();

        for pixel in &mut pixel_data {
            *pixel = (0.0, 1.0, 0.0);
        }

        exr_file
            .write_pixels_incremental(
                FrameBuffer::new(256, 64).insert_channels(&["R", "G", "B"], &pixel_data),
            ).unwrap();

        for pixel in &mut pixel_data {
            *pixel = (0.0, 0.0, 1.0);
        }

        exr_file
            .write_pixels_incremental(
                FrameBuffer::new(256, 64).insert_channels(&["R", "G", "B"], &pixel_data),
            ).unwrap();

        for pixel in &mut pixel_data {
            *pixel = (1.0, 1.0, 1.0);
        }

        exr_file
            .write_pixels_incremental(
                FrameBuffer::new(256, 64).insert_channels(&["R", "G", "B"], &pixel_data),
            ).unwrap();
    }

    // Read file from memory, and verify its contents
    {
        let mut exr_file = InputFile::from_slice(in_memory_buffer.get_ref()).unwrap();
        let (width, height) = exr_file.header().data_dimensions();

        // Make sure the image properties are the same.
        assert_eq!(width, 256);
        assert_eq!(height, 256);
        for channel_name in &["R", "G", "B"] {
            let channel = exr_file
                .header()
                .get_channel(channel_name)
                .expect(&format!("Didn't find channel {}.", channel_name));
            assert_eq!(channel.pixel_type, PixelType::FLOAT);
        }

        // Read in the pixel data in four chunks, verifying that each has
        // the data we expect.
        let mut pixel_data = vec![(0.0f32, 0.0f32, 0.0f32); 256 * 64];
        exr_file
            .read_pixels_partial(
                0,
                FrameBufferMut::new(256, 64)
                    .insert_channels(&[("R", 0.1), ("G", 0.1), ("B", 0.1)], &mut pixel_data),
            ).unwrap();
        for pixel in &pixel_data {
            assert_eq!(*pixel, (1.0, 0.0, 0.0));
        }

        exr_file
            .read_pixels_partial(
                64,
                FrameBufferMut::new(256, 64)
                    .insert_channels(&[("R", 0.1), ("G", 0.1), ("B", 0.1)], &mut pixel_data),
            ).unwrap();
        for pixel in &pixel_data {
            assert_eq!(*pixel, (0.0, 1.0, 0.0));
        }

        exr_file
            .read_pixels_partial(
                128,
                FrameBufferMut::new(256, 64)
                    .insert_channels(&[("R", 0.1), ("G", 0.1), ("B", 0.1)], &mut pixel_data),
            ).unwrap();
        for pixel in &pixel_data {
            assert_eq!(*pixel, (0.0, 0.0, 1.0));
        }

        exr_file
            .read_pixels_partial(
                192,
                FrameBufferMut::new(256, 64)
                    .insert_channels(&[("R", 0.1), ("G", 0.1), ("B", 0.1)], &mut pixel_data),
            ).unwrap();
        for pixel in &pixel_data {
            assert_eq!(*pixel, (1.0, 1.0, 1.0));
        }
    }
}
