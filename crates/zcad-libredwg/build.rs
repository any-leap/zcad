//! Build script for zcad-libredwg
//!
//! This script:
//! 1. Finds the LibreDWG library on the system
//! 2. Generates Rust bindings using bindgen

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-env-changed=LIBREDWG_DIR");
    println!("cargo:rerun-if-env-changed=VCPKG_ROOT");

    // Try to find libredwg
    let (include_paths, lib_path) = find_libredwg();

    // Link to libredwg
    if let Some(lib_path) = &lib_path {
        println!("cargo:rustc-link-search=native={}", lib_path.display());
    }
    
    // Library name varies by platform and installation method
    #[cfg(windows)]
    {
        // vcpkg installs as "libredwg.lib"
        if lib_path.as_ref().map(|p| p.join("libredwg.lib").exists()).unwrap_or(false) {
            println!("cargo:rustc-link-lib=libredwg");
        } else {
            // Some builds use "redwg"
            println!("cargo:rustc-link-lib=redwg");
        }
    }
    #[cfg(not(windows))]
    println!("cargo:rustc-link-lib=redwg");

    // Generate bindings
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Only generate bindings for dwg_ prefixed functions
        .allowlist_function("dwg_.*")
        .allowlist_function("dxf_.*")
        .allowlist_type("Dwg_.*")
        .allowlist_type("dwg_.*")
        .allowlist_type("Bit_Chain")
        .allowlist_var("DWG_.*")
        // Generate safer Rust types
        .derive_debug(true)
        .derive_default(true)
        .size_t_is_usize(true)
        // Block problematic types that cause issues
        .blocklist_type("max_align_t")
        .blocklist_type("__fsid_t");

    // Add include paths
    for include_path in &include_paths {
        builder = builder.clang_arg(format!("-I{}", include_path.display()));
    }

    let bindings = builder
        .generate()
        .expect("Unable to generate bindings for libredwg");

    // Write bindings to the OUT_DIR
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

/// Find LibreDWG library on the system
fn find_libredwg() -> (Vec<PathBuf>, Option<PathBuf>) {
    // First check for LIBREDWG_DIR environment variable
    if let Ok(dir) = env::var("LIBREDWG_DIR") {
        let path = PathBuf::from(&dir);
        let include = path.join("include");
        let lib = path.join("lib");
        if include.exists() && lib.exists() {
            println!("cargo:warning=Using LIBREDWG_DIR: {}", dir);
            // Check if headers are in a subdirectory
            let libredwg_subdir = include.join("libredwg");
            if libredwg_subdir.exists() {
                return (vec![include.clone(), libredwg_subdir], Some(lib));
            }
            return (vec![include], Some(lib));
        }
    }

    // Try VCPKG_ROOT environment variable
    if let Ok(vcpkg_root) = env::var("VCPKG_ROOT") {
        let vcpkg_path = PathBuf::from(&vcpkg_root);
        let installed = vcpkg_path.join("installed").join("x64-windows");
        let include = installed.join("include");
        let lib = installed.join("lib");
        let libredwg_include = include.join("libredwg");
        
        if libredwg_include.join("dwg.h").exists() {
            println!("cargo:warning=Found libredwg via VCPKG_ROOT: {}", vcpkg_root);
            return (vec![include, libredwg_include], Some(lib));
        }
    }

    // Try pkg-config on Unix-like systems
    #[cfg(not(windows))]
    {
        if let Ok(lib) = pkg_config::Config::new()
            .atleast_version("0.12")
            .probe("libredwg")
        {
            println!("cargo:warning=Found libredwg via pkg-config");
            let includes: Vec<PathBuf> = lib.include_paths.clone();
            let lib_path = lib.link_paths.first().cloned();
            return (includes, lib_path);
        }
    }

    // Try vcpkg on Windows
    #[cfg(windows)]
    {
        if let Ok(lib) = vcpkg::find_package("libredwg") {
            println!("cargo:warning=Found libredwg via vcpkg auto-detect");
            let mut includes: Vec<PathBuf> = lib.include_paths.clone();
            // Also add libredwg subdirectory if it exists
            for path in &lib.include_paths {
                let subdir = path.join("libredwg");
                if subdir.exists() {
                    includes.push(subdir);
                }
            }
            let lib_path = lib.link_paths.first().cloned();
            return (includes, lib_path);
        }
    }

    // Try common vcpkg installation paths on Windows
    #[cfg(windows)]
    {
        let home = env::var("USERPROFILE").unwrap_or_default();
        let common_vcpkg_paths = vec![
            PathBuf::from(&home).join("developer").join("vcpkg"),
            PathBuf::from(&home).join("vcpkg"),
            PathBuf::from("C:/vcpkg"),
            PathBuf::from("C:/src/vcpkg"),
        ];
        
        for vcpkg_path in common_vcpkg_paths {
            let installed = vcpkg_path.join("installed").join("x64-windows");
            let include = installed.join("include");
            let lib = installed.join("lib");
            let libredwg_include = include.join("libredwg");
            
            if libredwg_include.join("dwg.h").exists() {
                println!("cargo:warning=Found libredwg at: {}", vcpkg_path.display());
                return (vec![include, libredwg_include], Some(lib));
            }
        }
    }

    // Try common installation paths
    let common_paths = if cfg!(windows) {
        vec![
            PathBuf::from("C:/libredwg"),
            PathBuf::from("C:/Program Files/libredwg"),
            PathBuf::from("C:/msys64/mingw64"),
        ]
    } else if cfg!(target_os = "macos") {
        vec![
            PathBuf::from("/opt/homebrew"),
            PathBuf::from("/usr/local"),
        ]
    } else {
        vec![
            PathBuf::from("/usr"),
            PathBuf::from("/usr/local"),
        ]
    };

    for base in common_paths {
        let include = base.join("include");
        let lib = base.join("lib");
        
        // Check for dwg.h directly or in libredwg subdirectory
        if include.join("dwg.h").exists() {
            println!("cargo:warning=Found libredwg at: {}", base.display());
            return (vec![include], Some(lib));
        }
        
        let libredwg_subdir = include.join("libredwg");
        if libredwg_subdir.join("dwg.h").exists() {
            println!("cargo:warning=Found libredwg at: {}", base.display());
            return (vec![include, libredwg_subdir], Some(lib));
        }
    }

    println!("cargo:warning=LibreDWG not found! Please install it or set LIBREDWG_DIR");
    println!("cargo:warning=  Windows: vcpkg install libredwg or download from https://github.com/LibreDWG/libredwg/releases");
    println!("cargo:warning=  macOS: brew install libredwg");
    println!("cargo:warning=  Linux: apt install libredwg-dev or build from source");
    
    (vec![], None)
}
