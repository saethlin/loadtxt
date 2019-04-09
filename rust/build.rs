fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;
    cbindgen::generate_with_config(&crate_dir, config)
        .unwrap()
        .write_to_file("target/loadtxt.h");
}
