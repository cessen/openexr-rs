extern crate cmake;
extern crate gcc;

fn main() {
    // Build IlmBase
    let ilmbase_dst = cmake::Config::new("openexr/IlmBase")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    // Build OpenEXR
    let openexr_dst = cmake::Config::new("openexr/OpenEXR")
        .define("ILMBASE_PACKAGE_PREFIX", &ilmbase_dst)
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
    let exr_include_path = openexr_dst.join("include");

    // Build C wrapper for OpenEXR
    gcc::Config::new()
                .cpp(true)
                .include("c_wrapper")
                .include(&exr_include_path)
                .file("c_wrapper/cexr.cpp")
                .compile("libcexr.a");
}
