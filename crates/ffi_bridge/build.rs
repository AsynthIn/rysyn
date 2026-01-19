use std::env;
use std::path::PathBuf;

fn main() {
    // Generate C header with cbindgen
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let output_file = PathBuf::from(&crate_dir)
        .join("../../apps/juce_core/include")
        .join("rysyn_ffi.h");

    if let Ok(bindings) = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_language(cbindgen::Language::C)
        .with_include_guard("RYSYN_FFI_H")
        .with_pragma_once(true)
        .generate()
    {
        bindings.write_to_file(&output_file);
        println!("cargo:warning=Generated FFI header: {:?}", output_file);
    }
}
