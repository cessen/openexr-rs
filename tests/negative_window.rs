extern crate half;
extern crate openexr;
extern crate openexr_sys;

use half::f16;
use openexr::{FrameBuffer, FrameBufferMut, Header, InputFile, ScanlineOutputFile};
use std::fs::File;

// OpenEXR file data.
const DATA: &[u8] = include_bytes!("data/negative_window.exr");

#[test]
fn negative_window_read_multiple_channels() {
    let mut exr_file = InputFile::from_slice(DATA).unwrap();

    let (width, height) = exr_file.header().data_dimensions();
    let (x, y) = exr_file.header().data_origin();

    let zero = f16::from_f32(0.0f32);

    println!("Reading pixels from {},{},{}", "R", "G", "B");
    let mut pixel_data = vec![(zero, zero, zero); (width * height) as usize];

    // let's try a mismatched origin 0,0
    {
        let mut fb = FrameBufferMut::new(width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        assert!(exr_file.read_pixels(&mut fb).is_err());
    }

    // let's try a mismatched origin somewhere else
    {
        let mut fb = FrameBufferMut::new_with_origin(10, 11, width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        assert!(exr_file.read_pixels(&mut fb).is_err());
    }

    // and then the real thing
    {
        let mut fb = FrameBufferMut::new_with_origin(x, y, width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        exr_file.read_pixels(&mut fb).unwrap();
    }

    // we write the file back out with a different offset
    {
        let mut fb = FrameBuffer::new_with_origin(-8, -8, width, height);
        println!("Loading buffer as {}x{}", width, height);
        let mut file = File::create("target/negative_window_with_offset.exr")
            .expect("Could not create output file");
        let mut exr_file = ScanlineOutputFile::new(
            &mut file,
            &Header::new()
                .set_data_window(Header::box2i(-8, -8, width, height))
                .set_display_window(Header::box2i(0, 0, width -16, height - 16))
                .add_channel("R", openexr::PixelType::HALF)
                .add_channel("G", openexr::PixelType::HALF)
                .add_channel("B", openexr::PixelType::HALF),
        )
        .unwrap();

        fb.insert_channels(&["R", "G", "B"], &pixel_data);
        exr_file.write_pixels(&fb).unwrap();
    }
}

// with one channel only as well
#[test]
fn negative_window_read_single_channel() {
    let mut exr_file = InputFile::from_slice(DATA).unwrap();

    let (width, height) = exr_file.header().data_dimensions();
    let (x, y) = exr_file.header().data_origin();

    let zero = f16::from_f32(0.0f32);

    println!("Reading pixels from {},{},{}", "R", "G", "B");
    let mut pixel_data = vec![zero; (width * height) as usize];

    // let's try a mismatched origin in 0,0 (default)
    {
        let mut fb = FrameBufferMut::new(width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channel("R", 0.0, &mut pixel_data);
        assert!(exr_file.read_pixels(&mut fb).is_err());
    }

    // let's try a mismatched origin in 0,0 (default)
    {
        let mut fb = FrameBufferMut::new_with_origin(-1, -10, width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channel("R", 0.0, &mut pixel_data);
        assert!(exr_file.read_pixels(&mut fb).is_err());
    }

    // now the real deal
    {
        let mut fb = FrameBufferMut::new_with_origin(x, y, width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channel("R", 0.0, &mut pixel_data);
        exr_file.read_pixels(&mut fb).unwrap();
    }
}
