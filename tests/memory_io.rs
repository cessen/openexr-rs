extern crate openexr;

use std::io::Cursor;

use openexr::{FrameBuffer, FrameBufferMut, Header, InputFile, PixelType, ScanlineOutputFile};

#[test]
fn memory_io() {
    // Target memory for writing
    let mut in_memory_buffer = Cursor::new(Vec::<u8>::new());

    // Write file to memory
    {
        let pixel_data = vec![(0.82f32, 1.78f32, 0.21f32); 256 * 256];

        let mut exr_file = ScanlineOutputFile::new(
            &mut in_memory_buffer,
            Header::new()
                .set_resolution(256, 256)
                .add_channel("R", PixelType::FLOAT)
                .add_channel("G", PixelType::FLOAT)
                .add_channel("B", PixelType::FLOAT),
        )
        .unwrap();

        let mut fb = FrameBuffer::new(256, 256);
        fb.insert_channels(&["R", "G", "B"], &pixel_data);

        exr_file.write_pixels(&fb).unwrap();
    }

    // Read file from memory, and verify its contents
    {
        let mut pixel_data = vec![(0.0f32, 0.0f32, 0.0f32); 256 * 256];

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

        // Read in the pixel data.
        {
            let mut fb = FrameBufferMut::new(width, height);
            fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);

            exr_file.read_pixels(&mut fb).unwrap();
        }

        // Verify the data is what we expect
        for pixel in pixel_data {
            assert_eq!(pixel, (0.82, 1.78, 0.21));
        }
    }
}
