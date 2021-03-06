extern crate half;
extern crate openexr;
extern crate openexr_sys;

use half::f16;
use openexr::{FrameBuffer, FrameBufferMut, Header, InputFile, ScanlineOutputFile};
use std::fs::File;

// OpenEXR file data.
const NEGATIVE_OFFSET: &[u8] = include_bytes!("data/negative_window.exr");
const POSITIVE_OFFSET: &[u8] = include_bytes!("data/positive_window.exr");

/// Return the offset index of the data pixel at 0,0 with
/// reference to the data pixel at origin.
fn calc_origin_offset(image_width: u32, origin: (i32, i32)) -> isize {
    let (x, y) = origin;
    -(x as isize + y as isize * image_width as isize)
}

fn load_and_test_with_offset_window_read_multiple_channels(data: &[u8]) {
    let mut exr_file = InputFile::from_slice(data).unwrap();

    let (width, height) = exr_file.header().data_dimensions();
    let (x, y) = exr_file.header().data_origin();

    let zero = f16::from_f32(0.0f32);

    let mut pixel_data = vec![(zero, zero, zero); (width * height) as usize];

    let read_with_offset = |exr_file: &mut InputFile, pixel_data: &mut [_], ox, oy| {
        let mut fb = FrameBufferMut::new_with_origin(ox, oy, width, height);
        println!("Loading from buffer as {},{} {}x{}", ox, oy, width, height);
        fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], pixel_data);
        println!("Reading pixels from {},{},{}", "R", "G", "B");
        (
            exr_file.read_pixels(&mut fb),
            calc_origin_offset(width, (ox, oy)),
        )
    };

    // let's try a few mismatched origins
    assert!(read_with_offset(&mut exr_file, &mut pixel_data, 0, 0)
        .0
        .is_err());
    assert!(
        read_with_offset(&mut exr_file, &mut pixel_data, x - 1, y - 1)
            .0
            .is_err()
    );
    assert!(read_with_offset(&mut exr_file, &mut pixel_data, -x, -y)
        .0
        .is_err());
    // and then the real thing
    let (read_result, origin_offset) = read_with_offset(&mut exr_file, &mut pixel_data, x, y);
    assert!(read_result.is_ok());
    // check the pixel value at coordinates (0,0) of the data window if 0,0 is within the frame buffer
    if origin_offset >= 0 {
        assert!(f32::abs(f32::from(pixel_data[origin_offset as usize].0) - 0.5f32) < 0.0001f32);
    }

    // we write the file back out with a different offset
    {
        let mut fb = FrameBuffer::new_with_origin(-x, -y, width, height);
        println!("Loading buffer as {}x{}", width, height);
        let mut file =
            File::create("target/positive_window.exr").expect("Could not create output file");
        let mut exr_file = ScanlineOutputFile::new(
            &mut file,
            &Header::new()
                .set_data_window(Header::box2i(-x, -y, width, height))
                .set_display_window(Header::box2i(0, 0, width - 16, height - 16))
                .add_channel("R", openexr::PixelType::HALF)
                .add_channel("G", openexr::PixelType::HALF)
                .add_channel("B", openexr::PixelType::HALF),
        )
        .unwrap();

        fb.insert_channels(&["R", "G", "B"], &pixel_data);
        exr_file.write_pixels(&fb).unwrap();
    }
}

fn load_and_test_with_offset_window_read_single_channel(data: &[u8]) {
    let mut exr_file = InputFile::from_slice(data).unwrap();
    let (width, height) = exr_file.header().data_dimensions();
    let (x, y) = exr_file.header().data_origin();

    let zero = f16::from_f32(0.0f32);

    let mut pixel_data = vec![zero; (width * height) as usize];

    let mut read_with_offset = |ox: i32, oy: i32| {
        let mut fb = FrameBufferMut::new_with_origin(ox, oy, width, height);
        println!(
            "Loading from buffer as ({} {}) {}x{}",
            ox, oy, width, height
        );
        fb.insert_channel("R", 0.0, &mut pixel_data);
        println!("Reading pixels from {},{},{}", "R", "G", "B");
        exr_file.read_pixels(&mut fb)
    };
    // let's try a few mismatched origins
    assert!(read_with_offset(0, 0).is_err());
    assert!(read_with_offset(x - 1, y - 1).is_err());
    assert!(read_with_offset(-x, -y).is_err());
    // and then the correct one
    assert!(read_with_offset(x, y).is_ok());
}

#[test]
fn window_read_multiple_channels() {
    load_and_test_with_offset_window_read_multiple_channels(NEGATIVE_OFFSET);
    load_and_test_with_offset_window_read_multiple_channels(POSITIVE_OFFSET);
}

// with one channel only as well
#[test]
fn negative_window_read_single_channel() {
    load_and_test_with_offset_window_read_single_channel(NEGATIVE_OFFSET);
    load_and_test_with_offset_window_read_single_channel(POSITIVE_OFFSET);
}
