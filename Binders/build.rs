
// #[macro_use] 
// extern crate serde_json;
use std::{env, path::PathBuf};


pub const CORE_HEADER: &'static str = "def.h";
#[allow(unreachable_code)]
fn main(){
    return;
    let workspace = PathBuf::from(env::current_dir().unwrap()).join("BaseTypes/").as_path().display().to_string().replace("\\", "/");
    let export_header = format!("{}{}", workspace, CORE_HEADER);
    println!("cargo:rerun-if-changed={}",export_header);
    let bindings = bindgen::Builder::default()
        .header(&export_header)
        // .header(format!("{}/Runtime/Core/Private/CorePrivatePCH.h", UE_WORKSPACE))
        // .header(format!("{}/Runtime/Core/Public/CoreSharedPCH.h", UE_WORKSPACE))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    // let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(workspace + "/../src/cross_types.rs")
        .expect("Couldn't write bindings!");
}