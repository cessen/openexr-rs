extern crate cmake;

fn main() {
    // Build IlmBase
    let ilmbase_dst = cmake::build("openexr/IlmBase");
    let _ = cmake::Config::new("openexr/OpenEXR")
        .define("ILMBASE_PACKAGE_PREFIX", &ilmbase_dst)
        .build();
}
