extern crate pkg_config;
extern crate cmake;
extern crate gcc;

fn main() {
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
    let exr_include_path = openexr_dst.join("include");

    // Build C wrapper for OpenEXR
    gcc::Config::new()
                .cpp(true)
                .flag("-std=c++11")
                .include("c_wrapper")
                .include(&exr_include_path)
                .file("c_wrapper/cexr.cpp")
                .file("c_wrapper/memory_istream.cpp")
                .compile("libcexr.a");

    // Link all the libs from OpenEXR
    println!("cargo:rustc-link-lib=static=IlmImf-2_2");
    println!("cargo:rustc-link-lib=static=IlmImfUtil-2_2");
    println!("cargo:rustc-link-lib=static=IexMath-2_2");
    println!("cargo:rustc-link-lib=static=Iex-2_2");
    println!("cargo:rustc-link-lib=static=Imath-2_2");
    println!("cargo:rustc-link-lib=static=IlmThread-2_2");
    println!("cargo:rustc-link-lib=static=Half");

    // Find and link zlib, needed for OpenEXR
    pkg_config::probe_library("zlib").unwrap();
}
