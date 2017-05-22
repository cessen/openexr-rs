extern crate pkg_config;
extern crate gcc;

use std::env;
use std::path::PathBuf;

fn main() {
    // Find and link OpenEXR and IlmBase
    let include_paths = {
        let mut include_paths = Vec::new();

        let suffix = if let Ok(v) = env::var("OPENEXR_LIB_SUFFIX") {
            format!("-{}", v)
        } else {
            "".into()
        };

        if let Ok(path) = env::var("OPENEXR_DIR") {
            // There's an environment variable, so let's use that
            println!("cargo:rustc-link-search=native={}/lib", path);
            println!("cargo:rustc-link-lib=static=IlmImf{}", suffix);
            println!("cargo:rustc-link-lib=static=IlmImfUtil{}", suffix);
            include_paths.push(PathBuf::from(&format!("{}/include/OpenEXR", path)));
        } else {
            // There's no enviroment variable, so use pkgconfig to find
            // the libs
            let paths = pkg_config::Config::new()
                .atleast_version("2.0.0")
                .probe("OpenEXR")
                .map(|openexr_cfg| openexr_cfg.include_paths.clone())
                .map_err(|err| {
                    panic!("couldn't find OpenEXR: environment variable \
                            OPENEXR_DIR is unset and pkg-config failed: {}",
                           err)
                })
                .unwrap();

            include_paths.extend_from_slice(&paths);
        }

        if let Ok(path) = env::var("ILMBASE_DIR") {
            println!("cargo:rustc-link-search=native={}/lib", path);
            println!("cargo:rustc-link-lib=static=IexMath{}", suffix);
            println!("cargo:rustc-link-lib=static=Iex{}", suffix);
            println!("cargo:rustc-link-lib=static=Imath{}", suffix);
            println!("cargo:rustc-link-lib=static=IlmThread{}", suffix);
            println!("cargo:rustc-link-lib=static=Half");
            include_paths.push(PathBuf::from(&format!("{}/include/OpenEXR", path)));
        } else {
            let paths = pkg_config::Config::new()
                .atleast_version("2.0.0")
                .cargo_metadata(false) // OpenEXR already pulls in all the flags we need
                .probe("IlmBase")
                .map(|ilmbase_cfg| ilmbase_cfg.include_paths.clone())
                .map_err(|err| {
                    panic!("couldn't find IlmBase: environment variable \
                            ILMBASE_DIR is unset and pkg-config failed: {}",
                           err)
                })
                .unwrap();
            include_paths.extend_from_slice(&paths);
        }

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
    gcc.flag("-std=c++0x");
    for path in &include_paths {
        gcc.include(path);
    }
    gcc.file("c_wrapper/cexr.cpp")
        .file("c_wrapper/memory_istream.cpp")
        .compile("libcexr.a");
}
