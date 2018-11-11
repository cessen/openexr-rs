extern crate half;
extern crate openexr;

use half::f16;
use openexr::{FrameBufferMut, InputFile};

// OpenEXR file data.
const DATA: &[u8] = include_bytes!("data/negative_window.exr");

#[test]
fn negative_window_read_multiple_channels() {
    let mut exr_file = InputFile::from_slice(DATA).unwrap();

    let (width, height) = exr_file.header().data_dimensions();
    let openexr::Box2i {
        min: openexr_sys::CEXR_V2i { x, y },
        ..
    } = *exr_file.header().data_window();
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
}

// with one channel only as well
#[test]
fn negative_window_read_single_channel() {
    let mut exr_file = InputFile::from_slice(DATA).unwrap();

    let (width, height) = exr_file.header().data_dimensions();
    let openexr::Box2i {
        min: openexr_sys::CEXR_V2i { x, y },
        ..
    } = *exr_file.header().data_window();
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
