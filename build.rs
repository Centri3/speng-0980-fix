use std::env::var;
use std::path::Path;

fn main() {
    println!("cargo:rustc-link-search=include/bin");
    if var("TARGET").unwrap().starts_with("x86_64") {
        println!("cargo:rustc-link-lib=include/amd64/nvapi64");
    } else {
        println!("cargo:rustc-link-lib=include/x86/nvapi");
    }
    println!("cargo:rerun-if-changed=wrapper.h");

    bindgen::builder()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new(&var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
