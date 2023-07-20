use std::env::var;
use std::path::Path;

fn main() {
    println!("cargo:rustc-link-search=include/bin");
    println!("cargo:rerun-if-changed=wrapper.h");

    bindgen::builder()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new(&var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
