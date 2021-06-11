use pkg_config;
use std::env;
use std::path::Path;

fn main() {
    let mut link_path = None;
    let mut include_path = None;
    let env_cuda_path = env::var("CUDA_PATH").unwrap_or("".to_string());

    // First let's look through some default directories, if they exist
    let possible_paths = vec![Path::new(&env_cuda_path), Path::new("/usr/local/cuda")];
    for cuda_path in possible_paths {
        if cuda_path.is_dir() {
            let possible_link = if cuda_path.join("lib64").is_dir() {
                cuda_path.join("lib64")
            } else {
                cuda_path.join("lib").join("x64")
            };
            let possible_include = cuda_path.join("include");

            if possible_link.is_dir() && possible_include.is_dir() {
                link_path = Some(possible_link);
                include_path = Some(possible_include);
                break;
            }
        }
    }

    // If all else fails, try looking through `pkg-config`
    if include_path.is_none() {
        let packages = vec!["cuda", "cudart", "cuda-11.3", "cudart-11.3"];
        for package in packages {
            if let Ok(pkg) = pkg_config::probe_library(package) {
                assert!(pkg.link_paths.len() == 1);
                assert!(pkg.include_paths.len() == 1);
                link_path = Some(pkg.link_paths[0].clone());
                include_path = Some(pkg.include_paths[0].clone());
                break;
            }
        }
    }

    // Hopefully by this point we have it all figured out...
    if let (Some(include_path), Some(link_path)) = (&include_path, link_path) {
        println!("cargo:include={}", include_path.to_str().unwrap());
        println!(
            "cargo:rustc-link-search=native={}",
            link_path.to_str().unwrap()
        );
    } else {
        panic!("unable to find cuda libraries");
    }

    //-------------------------------------------------------------------------

    let libs_env = env::var("CUDA_LIBS").ok();
    let libs = match libs_env {
        Some(ref v) => v.split(':').collect(),
        None => vec!["cudart"],
    };

    let mode = if env::var_os("CUDA_STATIC").is_some() {
        "static"
    } else {
        "dylib"
    };

    for lib in libs {
        println!("cargo:rustc-link-lib={}={}", mode, lib);
    }

    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(feature = "generate")]
    {
        use std::path::PathBuf;
        let bindings = bindgen::Builder::default()
            .rust_target(bindgen::RustTarget::Stable_1_40)
            .raw_line(
                r"
//! Defines the FFI for CUDA cuBLAS.
//!
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
            ",
            )
            .ctypes_prefix("::libc")
            .size_t_is_usize(true)
            .clang_arg("-I")
            .clang_arg(include_path.unwrap().to_str().unwrap())
            .header("wrapper.h")
            .rustified_non_exhaustive_enum("cuda.*")
            .whitelist_function("cu.*")
            .whitelist_var("CUDA.*")
            .whitelist_type("[Cc][Uu].*")
            .default_alias_style(bindgen::AliasVariation::TypeAlias)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .rustfmt_bindings(true)
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from("src").join("generated.rs");
        bindings
            .write_to_file(out_path)
            .expect("Couldn't write bindings!");
    }
}
