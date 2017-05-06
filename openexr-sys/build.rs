extern crate pkg_config;
extern crate cmake;
extern crate gcc;

use std::env;

fn main() {
    let include_paths = if env::var("USE_SYSTEM_OPENEXR").map(|x| x != "0").unwrap_or(false) {
        // We don't take linker flags from IlmBase because OpenEXR's subsume them.
        let ilm = pkg_config::Config::new().cargo_metadata(false).probe("IlmBase").unwrap();
        let exr = pkg_config::probe_library("OpenEXR").unwrap();
        let mut include_paths = ilm.include_paths.clone();
        include_paths.extend_from_slice(&exr.include_paths);
        include_paths
    } else {
        // Build IlmBase
        let ilmbase_dst = cmake::Config::new("openexr/IlmBase")
            .define("BUILD_SHARED_LIBS", "OFF")
            .build();
        println!("cargo:rustc-link-search=native={}", ilmbase_dst.join("lib").display());

        // Build OpenEXR
        let openexr_dst = cmake::Config::new("openexr/OpenEXR")
            .define("ILMBASE_PACKAGE_PREFIX", &ilmbase_dst)
            .define("BUILD_SHARED_LIBS", "OFF")
            .build();
        println!("cargo:rustc-link-search=native={}", openexr_dst.join("lib").display());

        // Link all the libs from OpenEXR
        println!("cargo:rustc-link-lib=static=IlmImf-2_2");
        println!("cargo:rustc-link-lib=static=IlmImfUtil-2_2");
        println!("cargo:rustc-link-lib=static=IexMath-2_2");
        println!("cargo:rustc-link-lib=static=Iex-2_2");
        println!("cargo:rustc-link-lib=static=Imath-2_2");
        println!("cargo:rustc-link-lib=static=IlmThread-2_2");
        println!("cargo:rustc-link-lib=static=Half");

        vec![openexr_dst.join("include")]
    };

    // Build C wrapper for OpenEXR
    let mut gcc = gcc::Config::new();
    gcc.cpp(true)
        .flag("-std=c++11")
        .include("c_wrapper");
    for path in &include_paths {
        gcc.include(path);
    }
    gcc.file("c_wrapper/cexr.cpp")
        .file("c_wrapper/memory_istream.cpp")
        .compile("libcexr.a");

    // Find and link zlib, needed for OpenEXR
    pkg_config::probe_library("zlib").unwrap();
}
