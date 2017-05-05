extern crate openexr;

use std::path::Path;

use openexr::{FrameBuffer, InputFile};

fn main() {
    let re = InputFile::from_file(Path::new("/tmp/test.exr")).unwrap();
    let window = re.data_window();
    let width = window.max.x - window.min.x + 1;
    let height = window.max.y - window.min.y + 1;

    let mut pixel_data: Vec<(f32, f32, f32)> = vec![(0.0, 0.0, 0.0); (width*height) as usize];

    {
        let mut fb = {
            // Create the frame buffer
            let mut fb = FrameBuffer::new(width as usize, height as usize);
            fb.insert_pixels(
                &[("R", 0.0), ("G", 0.0), ("B", 0.0)],
                &mut pixel_data
            );
            fb
        };

        re.read_pixels(&mut fb).unwrap();
    }

    for pixel in pixel_data {
        assert_eq!(pixel, (0.82, 1.78, 0.21));
    }
}
