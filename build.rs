extern crate bindgen;
extern crate cc;
extern crate pkg_config;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    bindings();
    compile();
}

// From https://github.com/Boscop/web-view
fn compile() {
    let mut build = cc::Build::new();

    let target = env::var("TARGET").unwrap();

    build
        .include("c/webview.h")
        .flag_if_supported("-std=c11")
        .flag_if_supported("-w");

    if env::var("DEBUG").is_err() {
        build.define("NDEBUG", None);
    } else {
        build.define("DEBUG", None);
    }

    if target.contains("windows") {
        build.define("UNICODE", None);

        if cfg!(feature = "edge") {
            build
                .file("webview_edge.cpp")
                .flag_if_supported("/std:c++17");

            for &lib in &["windowsapp", "user32", "gdi32", "ole32"] {
                println!("cargo:rustc-link-lib={}", lib);
            }
        } else {
            build.file("c/webview_mshtml.c");

            for &lib in &["ole32", "comctl32", "oleaut32", "uuid", "gdi32", "user32"] {
                println!("cargo:rustc-link-lib={}", lib);
            }
        }
    } else if target.contains("linux") || target.contains("bsd") {
        pkg_config::Config::new()
            .atleast_version("2.8")
            .probe("webkit2gtk-4.0")
            .unwrap();
    } else if target.contains("apple") {
        build
            .file("c/webview_cocoa.c")
            .define("OBJC_OLD_DISPATCH_PROTOTYPES", "1")
            .flag("-x")
            .flag("objective-c");
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=WebKit");
    } else {
        panic!("unsupported target");
    }

    build.compile("webview");
}

fn bindings() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.

    // Tell cargo to invalidate the built crate whenever the wrapper changes

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
