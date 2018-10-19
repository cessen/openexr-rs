extern crate half;
extern crate openexr;

use half::f16;
use openexr::{FrameBufferMut, InputFile};

// OpenEXR file data.
const DATA: &[u8] = include_bytes!("data/negative_window.exr");

#[test]
fn negative_window_read() {
    let mut exr_file = InputFile::from_slice(DATA).unwrap();

    let (width, height) = exr_file.header().data_dimensions();
    let zero = f16::from_f32(0.0f32);

    println!("Reading pixels from {},{},{}", "R", "G", "B");
    let mut pixel_data = vec![(zero, zero, zero); (width * height) as usize];

    {
        let mut fb = FrameBufferMut::new(width, height);
        println!("Loading buffer as {}x{}", width, height);
        fb.insert_channels(&[("R", 0.0), ("G", 0.0), ("B", 0.0)], &mut pixel_data);
        exr_file.read_pixels(&mut fb).unwrap();
    }
}
