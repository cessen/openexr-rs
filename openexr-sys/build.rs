extern crate pkg_config;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    // Find and link OpenEXR and IlmBase
    let include_paths = if let Ok(path) = env::var("OPENEXR_DIR") {
        // There's an environment variable, so let's use that
        println!("cargo:rustc-link-search=native={}/lib", path);
        println!("cargo:rustc-link-lib=static=IlmImf-2_2");
        println!("cargo:rustc-link-lib=static=IlmImfUtil-2_2");
        println!("cargo:rustc-link-lib=static=IexMath-2_2");
        println!("cargo:rustc-link-lib=static=Iex-2_2");
        println!("cargo:rustc-link-lib=static=Imath-2_2");
        println!("cargo:rustc-link-lib=static=IlmThread-2_2");
        println!("cargo:rustc-link-lib=static=Half");
        vec![PathBuf::from(&format!("{}/include", path))]
    } else {
        // There's no enviroment variable, so use pkgconfig to find
        // the libs
        let openexr_include_paths = pkg_config::Config::new()
            .atleast_version("2.0.0")
            .probe("OpenEXR")
            .map(|openexr_cfg| openexr_cfg.include_paths.clone())
            .map_err(|err| {
                         panic!("couldn't find OpenEXR: environment variable \
                OPENEXR_DIR is unset and pkg-config failed: {}",
                                err)
                     })
            .unwrap();

        let ilmbase_include_paths = pkg_config::Config::new()
            .atleast_version("2.0.0")
            .cargo_metadata(false)
            .probe("IlmBase")
            .map(|ilmbase_cfg| ilmbase_cfg.include_paths.clone())
            .map_err(|err| {
                         panic!("couldn't find IlmBase: environment variable \
                OPENEXR_DIR is unset and pkg-config failed: {}",
                                err)
                     })
            .unwrap();

        let mut include_paths = vec![];
        include_paths.extend_from_slice(&ilmbase_include_paths);
        include_paths.extend_from_slice(&openexr_include_paths);
        include_paths
    };

    // Find and link zlib, needed for OpenEXR
    // Use environment variable if it exists, and otherwise use pkgconfig.
    if let Ok(path) = env::var("ZLIB_DIR") {
        println!("cargo:rustc-link-search=native={}/lib", path);
        println!("cargo:rustc-link-lib=static=zlibstatic");
    } else if let Err(err) = pkg_config::probe_library("zlib") {
        panic!("couldn't find zlib: environment variable ZLIB_DIR is unset \
            and pkg-config failed: {}",
               err);
    }

    // Build C wrapper for OpenEXR
    let mut gcc = gcc::Config::new();
    gcc.cpp(true).include("c_wrapper");
    #[cfg(target_env = "msvc")]
    gcc.flag("/std:c++14");
    #[cfg(not(target_env = "msvc"))]
    gcc.flag("-std=c++14");
    for path in &include_paths {
        gcc.include(path);
    }
    gcc.file("c_wrapper/cexr.cpp")
        .file("c_wrapper/memory_istream.cpp")
        .compile("libcexr.a");
}
