extern crate cmake;

fn main() {
    // Build IlmBase
    let ilmbase_dst = cmake::Config::new("openexr/IlmBase")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
    let _ = cmake::Config::new("openexr/OpenEXR")
        .define("ILMBASE_PACKAGE_PREFIX", &ilmbase_dst)
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
}
