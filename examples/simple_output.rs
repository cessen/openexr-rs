extern crate openexr;

use std::path::Path;

use openexr::{ExrWriter, ExrWriterBuilder, Channel, PixelType};

fn main() {
    let mut wr = ExrWriterBuilder::new(Path::new("/tmp/test.exr"))
        .display_window((0,0), (256, 256))
        .data_window((0,0), (256, 256))
        .insert_channel("R", Channel::with_type(PixelType::F32))
        .insert_channel("G", Channel::with_type(PixelType::F32))
        .insert_channel("B", Channel::with_type(PixelType::F32))
        .insert_channel("A", Channel::with_type(PixelType::F32))
        .open();
}
