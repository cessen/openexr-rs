extern crate half;

use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use half::f16;


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("data_type_offsets.rs");
    let mut f = File::create(&dest_path).unwrap();

    let _ = write_data_type_offsets(&mut f);
}

// Calculates the internal memory offsets for various structured data types.
//
// The offsets are then used to implement traits that need that information.
// We can't just assume the data offsets, because Rust doesn't define data
// layout e.g. for tuples.
fn write_data_type_offsets(f: &mut File) -> io::Result<()> {
    // Memory offsets for a (f32, f32)
    let t2_f32 = {
        let f2 = (0.0f32, 0.0f32);
        let f2_start = &f2 as *const _ as usize;
        let f2_0 = &f2.0 as *const _ as usize;
        let f2_1 = &f2.1 as *const _ as usize;
        (f2_0 - f2_start, f2_1 - f2_start)
    };
    f.write_all(format!("const T2_F32: (usize, usize) = ({}, {});\n",
                           t2_f32.0,
                           t2_f32.1)
                           .as_bytes())?;

    // Memory offsets for a (f32, f32, f32)
    let t3_f32 = {
        let f3 = (0.0f32, 0.0f32, 0.0f32);
        let f3_start = &f3 as *const _ as usize;
        let f3_0 = &f3.0 as *const _ as usize;
        let f3_1 = &f3.1 as *const _ as usize;
        let f3_2 = &f3.2 as *const _ as usize;
        (f3_0 - f3_start, f3_1 - f3_start, f3_2 - f3_start)
    };
    f.write_all(format!("const T3_F32: (usize, usize, usize) = ({}, {}, {});\n",
                           t3_f32.0,
                           t3_f32.1,
                           t3_f32.2)
                           .as_bytes())?;

    // Memory offsets for a (f32, f32, f32, f32)
    let t4_f32 = {
        let f4 = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let f4_start = &f4 as *const _ as usize;
        let f4_0 = &f4.0 as *const _ as usize;
        let f4_1 = &f4.1 as *const _ as usize;
        let f4_2 = &f4.2 as *const _ as usize;
        let f4_3 = &f4.3 as *const _ as usize;
        (f4_0 - f4_start, f4_1 - f4_start, f4_2 - f4_start, f4_3 - f4_start)
    };
    f.write_all(format!("const T4_F32: (usize, usize, usize, usize) = ({}, {}, {}, {});\n",
                           t4_f32.0,
                           t4_f32.1,
                           t4_f32.2,
                           t4_f32.3)
                           .as_bytes())?;

    // Memory offsets for a (f32, f32)
    let t2_f16 = {
        let f2 = (f16::from_f32(0.0), f16::from_f32(0.0));
        let f2_start = &f2 as *const _ as usize;
        let f2_0 = &f2.0 as *const _ as usize;
        let f2_1 = &f2.1 as *const _ as usize;
        (f2_0 - f2_start, f2_1 - f2_start)
    };
    f.write_all(format!("const T2_F16: (usize, usize) = ({}, {});\n",
                           t2_f16.0,
                           t2_f16.1)
                           .as_bytes())?;

    // Memory offsets for a (f16, f16, f16)
    let t3_f16 = {
        let f3 = (f16::from_f32(0.0), f16::from_f32(0.0), f16::from_f32(0.0));
        let f3_start = &f3 as *const _ as usize;
        let f3_0 = &f3.0 as *const _ as usize;
        let f3_1 = &f3.1 as *const _ as usize;
        let f3_2 = &f3.2 as *const _ as usize;
        (f3_0 - f3_start, f3_1 - f3_start, f3_2 - f3_start)
    };
    f.write_all(format!("const T3_F16: (usize, usize, usize) = ({}, {}, {});\n",
                           t3_f16.0,
                           t3_f16.1,
                           t3_f16.2)
                           .as_bytes())?;

    // Memory offsets for a (f16, f16, f16, f16)
    let t4_f16 = {
        let f4 = (f16::from_f32(0.0), f16::from_f32(0.0), f16::from_f32(0.0), f16::from_f32(0.0));
        let f4_start = &f4 as *const _ as usize;
        let f4_0 = &f4.0 as *const _ as usize;
        let f4_1 = &f4.1 as *const _ as usize;
        let f4_2 = &f4.2 as *const _ as usize;
        let f4_3 = &f4.3 as *const _ as usize;
        (f4_0 - f4_start, f4_1 - f4_start, f4_2 - f4_start, f4_3 - f4_start)
    };
    f.write_all(format!("const T4_F16: (usize, usize, usize, usize) = ({}, {}, {}, {});\n",
                           t4_f16.0,
                           t4_f16.1,
                           t4_f16.2,
                           t4_f16.3)
                           .as_bytes())?;

    Ok(())
}
