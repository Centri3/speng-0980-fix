use std::env::var;
use std::error::Error;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");

    if var("TARGET").unwrap().starts_with("x86_64") {
        println!("cargo:rustc-link-search=include/amd64");
        println!("cargo:rustc-link-lib=nvapi64");
    } else {
        println!("cargo:rustc-link-search=include/x86");
        println!("cargo:rustc-link-lib=nvapi");
    }

    bindgen::builder()
        .header("wrapper.h")
        .derive_default(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(Path::new(&var("OUT_DIR").unwrap()).join("bindings.rs"))
        .expect("Couldn't write bindings!");
    Ok(())
}
